// Rebuild morc's vendored React Flow bundle (ui/vendor/reactflow) from entry.js with the pinned packages in node_modules.
import { build } from 'esbuild';
await build({ entryPoints: ['entry.js'], bundle: true, format: 'esm', minify: true, outfile: 'reactflow.bundle.js',
  define: { 'process.env.NODE_ENV': '"production"' }, legalComments: 'none' });
