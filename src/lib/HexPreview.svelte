<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import layoutData from "../../reference/layout.json";

  let canvas: HTMLCanvasElement;
  let unlisten: (() => void) | undefined;
  const SIZE = 320; // display pixels

  onMount(async () => {
    unlisten = await listen<number[][][]>("preview", ({ payload }) => {
      drawPreview(payload);
    });

    // Draw an initial blank preview grid
    const blank = Array(32).fill(null).map(() => Array(32).fill([0, 0, 0]));
    drawPreview(blank);
  });

  onDestroy(() => {
    unlisten?.();
  });

  function drawPreview(px: number[][][]) {
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.clearRect(0, 0, SIZE, SIZE);

    const cellSize = SIZE / 32;
    const radius = cellSize * 0.35; // 3.5px dot

    // Add subtle background glow behind the hexagon
    ctx.shadowBlur = 15;
    ctx.shadowColor = "rgba(0, 120, 255, 0.05)";

    for (let y = 0; y < 32; y++) {
      for (let x = 0; x < 32; x++) {
        const lamp = layoutData[y * 32 + x];
        if (lamp > 0) {
          const cx = x * cellSize + cellSize / 2;
          const cy = y * cellSize + cellSize / 2;
          const rgb = px[y]?.[x] || [0, 0, 0];
          const isOff = rgb[0] === 0 && rgb[1] === 0 && rgb[2] === 0;

          ctx.beginPath();
          ctx.arc(cx, cy, radius, 0, 2 * Math.PI);
          
          if (isOff) {
            ctx.shadowBlur = 0;
            ctx.fillStyle = "rgba(45, 45, 55, 0.4)";
          } else {
            // Apply slight glow to lit LEDs
            ctx.shadowBlur = 6;
            ctx.shadowColor = `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, 0.5)`;
            ctx.fillStyle = `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`;
          }
          
          ctx.fill();
        }
      }
    }
  }
</script>

<div class="canvas-container">
  <canvas bind:this={canvas} width={SIZE} height={SIZE}></canvas>
</div>

<style>
  .canvas-container {
    display: flex;
    justify-content: center;
    align-items: center;
    background: radial-gradient(circle at center, #16161d, #0b0b0e);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 20px;
    padding: 20px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.03);
    width: fit-content;
    margin: 0 auto;
  }
  canvas {
    display: block;
    background: transparent;
  }
</style>
