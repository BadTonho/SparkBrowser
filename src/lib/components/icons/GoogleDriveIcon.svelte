<script lang="ts">
  let { size = 32 }: { size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true" focusable="false">
  <path fill="#00832d" d="M13.4 3h5.7L24 11.8l-2.9 5-7.7-13.4Z" />
  <path fill="#ffba00" d="M7.7 3h5.7L5.7 16.8l-2.9 5L0 16.8 7.7 3Z" />
  <path fill="#0066da" d="M5.7 16.8h15.4L18.2 22H2.8l2.9-5.2Z" />
</svg>
