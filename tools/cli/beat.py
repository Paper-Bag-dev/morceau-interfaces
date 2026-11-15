# morc: feed the shared /beat (cards glow, visualizers move) from system audio. Usage: morc beat [--demo] [--dance] [--secs N] [--fps 25] | morc beat --off
"""System audio (Windows: WASAPI loopback via pyaudiowpatch; macOS: a loopback input such as BlackHole via sounddevice) -> level, 8 bands, beat pulse, hue -> `@shared /beat`
through one persistent `morc mcp` process (no process per frame). --demo = a synthetic 120 BPM track, labelled as such."""
import json, math, os, subprocess, sys, time
import numpy as np

MORC = os.environ.get("MORC_BIN", "morc")
args = sys.argv[1:]
fps = float(args[args.index("--fps") + 1]) if "--fps" in args else 25.0
secs = float(args[args.index("--secs") + 1]) if "--secs" in args else 0.0

mcp = subprocess.Popen([MORC, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
rid = 0
def rpc(method, params, wait=False):
    global rid
    rid += 1
    mcp.stdin.write(json.dumps({"jsonrpc": "2.0", "id": rid, "method": method, "params": params}) + "\n"); mcp.stdin.flush()
    if wait: return mcp.stdout.readline()
def push(beat):
    rpc("tools/call", {"name": "patch", "arguments": {"card": "@shared", "data": {"/beat": beat}}})
    mcp.stdout.readline()   # one reply per call keeps the pipe from filling

rpc("initialize", {"protocolVersion": "2025-06-18"}, wait=True)
if "--off" in args:
    push({"on": False}); sys.exit(0)

def demo_frames():
    t0 = time.time()
    while True:
        t = time.time() - t0; ph = (t * 2.0) % 1.0           # 120 BPM
        kick = math.exp(-ph * 9); hat = math.exp(-((t * 4.0) % 1.0) * 14) * .5
        bands = [kick, kick * .8, .4 + .3 * math.sin(t * 3), .35 + .3 * math.sin(t * 5 + 1), hat, hat * .9, .3 + .2 * math.sin(t * 7), hat * .7]
        yield min(1, .25 + kick * .7), bands, ph < 0.06

def capture():
    """A read(n) -> float32 mono array and the sample rate. Windows: the default output's loopback. macOS: no system loopback
    exists, so the first input whose name has "blackhole" or "loopback" (install BlackHole and route output through it)."""
    if sys.platform == "win32":
        import pyaudiowpatch as pa
        p = pa.PyAudio(); dev = p.get_default_wasapi_loopback()
        rate, ch = int(dev["defaultSampleRate"]), dev["maxInputChannels"]; n = int(rate / fps)
        st = p.open(format=pa.paFloat32, channels=ch, rate=rate, input=True, input_device_index=dev["index"], frames_per_buffer=n)
        return (lambda: np.frombuffer(st.read(n, exception_on_overflow=False), dtype=np.float32).reshape(-1, ch).mean(1)), rate
    import sounddevice as sd
    dev = next((i for i, d in enumerate(sd.query_devices()) if d["max_input_channels"] > 0 and any(k in d["name"].lower() for k in ("blackhole", "loopback"))), None)
    if dev is None:
        raise ImportError("no loopback input (install BlackHole and send the output through it)")
    rate = int(sd.query_devices(dev)["default_samplerate"]); n = int(rate / fps)
    st = sd.InputStream(device=dev, channels=2, samplerate=rate, blocksize=n, dtype="float32"); st.start()
    return (lambda: st.read(n)[0].mean(1)), rate

def audio_frames():
    read, rate = capture()
    edges = np.geomspace(40, 12000, 9); avg = 1e-4; agc = 1e-3
    while True:
        x = read()
        f = np.abs(np.fft.rfft(x * np.hanning(len(x)))); fr = np.fft.rfftfreq(len(x), 1 / rate)
        bands = np.array([f[(fr >= a) & (fr < b)].mean() if ((fr >= a) & (fr < b)).any() else 0 for a, b in zip(edges, edges[1:])])
        agc = max(agc * .995, bands.max(), 1e-4); bands = np.clip(bands / agc, 0, 1)
        low = bands[:2].mean(); hit = low > avg * 1.35 and low > .25; avg = avg * .9 + low * .1
        yield float(min(1, math.sqrt((x ** 2).mean()) * 6)), bands.tolist(), hit

try:
    frames = demo_frames() if "--demo" in args else audio_frames()
except ImportError as e:
    need = "py -m pip install pyaudiowpatch" if sys.platform == "win32" else "pip3 install sounddevice, plus BlackHole as the output"
    print(f"system audio unavailable ({e}); needs: {need}. Or try `morc beat --demo`"); push({"on": False}); sys.exit(1)

def call(name, arguments):
    rpc("tools/call", {"name": name, "arguments": arguments})
    try: return json.loads(json.loads(mcp.stdout.readline())["result"]["content"][0]["text"])
    except Exception: return {}
# The window dance: every shown card's WINDOW hops round its home spot on each beat (a plate transform is clipped by the window).
homes = {}
if "--dance" in args:   # opt-in: it moves every card window
    for cd in (call("card", {}).get("cards") or []):
        if cd.get("state") == "shown" and cd.get("rect"): homes[cd["id"]] = cd["rect"][:2]
STEPS = [(0, -16), (14, 0), (0, 10), (-14, 0)]
def dance(n):
    for i, (cid, (x, y)) in enumerate(homes.items()):
        dx, dy = STEPS[(n + i) % 4]
        call("card", {"card": cid, "set": {"at": [x + dx, y + dy]}})
def home():
    for cid, (x, y) in homes.items(): call("card", {"card": cid, "set": {"at": [x, y]}})
nbeat = 0
print("beat: " + ("DEMO (synthetic 120 BPM, not your audio)" if "--demo" in args else "system audio") + ", Ctrl+C to stop")
pulse, hue, sw, t_end = 0.0, 0.0, 1, time.time() + secs if secs else float("inf")
try:
    for level, bands, hit in frames:
        if time.time() > t_end: break
        if hit:
            pulse, sw, hue = 1.0, -sw, hue + 40
            if homes: nbeat += 1; dance(nbeat)
        pulse *= 0.78; hue = (hue + 1.5 + level * 4) % 360
        push({"on": True, "l": round(level, 3), "p": round(pulse, 3), "h": round(hue), "sw": sw, "bands": [round(b, 3) for b in bands],
              "demo": "--demo" in args})
        time.sleep(1 / fps if "--demo" in args else 0)
finally:
    push({"on": False})
    if homes: home()
