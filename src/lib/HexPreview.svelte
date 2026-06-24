<script lang="ts">
  // T7: render 556-LED hexagon from "preview" events
  // Receives: { px: [[r,g,b], ...] } — 32x32 grid, row-major
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  let canvas: HTMLCanvasElement;
  let unlisten: (() => void) | undefined;
  const SIZE = 320; // display pixels

  onMount(async () => {
    unlisten = await listen<{ px: number[][][] }>("preview", ({ payload }) => {
      drawPreview(payload.px);
    });
  });

  onDestroy(() => unlisten?.());

  function drawPreview(_px: number[][][]) {
    // TODO T7: draw hexagon preview
  }
</script>

<canvas bind:this={canvas} width={SIZE} height={SIZE} />
