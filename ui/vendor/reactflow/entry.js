// One pinned ESM bundle for morc's flowmap component: React + ReactDOM + React Flow + dagre (layout). Built once with esbuild, vendored.
export { createElement, useState, useEffect, useMemo, useCallback, useRef } from 'react';
export { createRoot } from 'react-dom/client';
export { ReactFlow, ReactFlowProvider, Background, Controls, MiniMap, MarkerType, useReactFlow, useNodesState, useEdgesState, Handle, Position } from '@xyflow/react';
import dagre from '@dagrejs/dagre';
export { dagre };
