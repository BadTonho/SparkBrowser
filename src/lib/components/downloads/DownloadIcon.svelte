<script lang="ts">
  let { name, size = 16 }: { name: string; size?: number } = $props();
  const paths: Record<string, string> = {
    search: "m21 21-4.5-4.5M19 10.5a8.5 8.5 0 1 1-17 0 8.5 8.5 0 0 1 17 0",
    trash: "M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7m4-7v7",
    close: "m6 6 12 12M18 6 6 18",
    pause: "M8 5v14M16 5v14",
    resume: "m8 4 12 8-12 8Z",
    restart: "M3 10a9 9 0 1 1 2 8M3 4v6h6",
    source: "M14 3h7v7m0-7L10 14M10 3H4v17h17v-6",
    completed: "m5 12 4 4L19 6",
    error: "M12 4 2 21h20ZM12 9v5m0 3v.1",
    processing: "M12 3a9 9 0 1 1-9 9",
    file: "M14 2H5v20h14V7Zm0 0v5h5",
    document: "M14 2H5v20h14V7Zm0 0v5h5M8 12h8m-8 4h6",
    image: "M3 3h18v18H3Zm0 13 5-5 5 5 3-3 5 5M16 7v.1",
    video: "M2 5h14v14H2Zm14 5 6-4v12l-6-4",
    audio: "M9 18V5l12-2v13M9 8l12-2M9 18c0 2-6 4-6 1s6-4 6-1m12-2c0 2-6 4-6 1s6-4 6-1",
    archive: "M14 2H5v20h14V7Zm0 0v5h5M10 3v2m0 2v2m0 2v2m-1 3h3v3H9Z",
    application: "M3 3h18v18H3ZM3 8h18M6 5.5h.1m3 0h.1M8 12l-3 3 3 3m8-6 3 3-3 3",
  };
</script>
<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false"><path d={paths[name] ?? paths.file} /></svg>
