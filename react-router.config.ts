import type { Config } from "@react-router/dev/config";

export default {
  // GitHub Pages is static hosting, so generate a browser-only SPA build.
  ssr: false,
  // Project Pages serves this repository from https://<user>.github.io/2048-ai/.
  basename: "/2048-ai/",
} satisfies Config;
