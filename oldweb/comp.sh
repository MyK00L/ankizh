#!/bin/sh
# --noUncheckedIndexedAccess
tsc --noUnusedParameters --noUnusedLocals --noImplicitReturns --noImplicitOverride --allowUnusedLabels false --allowUnreachableCode false --strict js.ts
# tsc --noUnusedParameters --noImplicitReturns --noImplicitOverride --allowUnusedLabels false --allowUnreachableCode false --strict js.ts
# esbuild js.js --minify --outfile=minified.js

