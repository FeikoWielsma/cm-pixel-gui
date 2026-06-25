<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import layoutData from "../../reference/layout.json";

  let { 
    interactive = false, 
    path = "", 
    settings = null, 
    status = null,
    onchange 
  } = $props<{
    interactive?: boolean;
    path?: string;
    settings?: any;
    status?: any;
    onchange?: (zoom: number, panX: number, panY: number) => void;
  }>();

  let canvas: HTMLCanvasElement;
  let unlisten: (() => void) | undefined;
  const SIZE = 320; // display pixels

  // Local state for zoom and pan when interactive
  let zoom = $state(1.0);
  let panX = $state(0.0);
  let panY = $state(0.0);

  // Real image dimensions for arrow keys 1px adjustments
  let imgWidth = $state(1000);
  let imgHeight = $state(1000);

  // Loading state when files are importing/converting
  let frontendLoaded = $state(false);
  let backendLoaded = $derived(status?.loaded_path === path);
  let isLoading = $derived(
    interactive && path ? (!frontendLoaded || !backendLoaded) : false
  );

  // Sync state when active image path changes
  let initialParams = $derived(
    settings?.image_parameters?.[path] || { zoom: 1.0, pan_x: 0.0, pan_y: 0.0 }
  );

  $effect(() => {
    if (interactive && path) {
      frontendLoaded = false;
      // Use untrack so this effect only triggers when interactive or path changes.
      // Otherwise, dragging triggers this effect and resets local state repeatedly.
      const params = untrack(() => initialParams);
      zoom = params.zoom;
      // We scale the normalized pan offset (-0.5 to 0.5) to a pixel offset based on the SIZE of the canvas
      panX = params.pan_x * SIZE;
      panY = params.pan_y * SIZE;

      // Load original image dimensions untracked
      untrack(() => {
        const img = new window.Image();
        img.src = convertFileSrc(path);
        img.onload = () => {
          imgWidth = img.naturalWidth;
          imgHeight = img.naturalHeight;
          frontendLoaded = true;
        };
        img.onerror = () => {
          frontendLoaded = true;
        };
      });
    } else {
      frontendLoaded = false;
    }
  });

  // Calculate clamp limits based on the canvas size and aspect ratio
  let aspectRatio = $derived(imgWidth / imgHeight);
  let maxPanX = $derived(
    aspectRatio > 1.0 ? 160 * (aspectRatio * zoom - 1) : 160 * (zoom - 1)
  );
  let maxPanY = $derived(
    aspectRatio < 1.0 ? 160 * ((1.0 / aspectRatio) * zoom - 1) : 160 * (zoom - 1)
  );

  // Pointer state
  let isDragging = false;
  let startX = 0;
  let startY = 0;
  let startPanX = 0;
  let startPanY = 0;
  let debounceTimeout: any;

  function debouncedSyncSettings() {
    clearTimeout(debounceTimeout);
    debounceTimeout = setTimeout(() => {
      onchange?.(zoom, panX / SIZE, panY / SIZE);
    }, 50);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!interactive) return;

    let handled = false;
    const s_base = Math.min(imgWidth, imgHeight) || 1000;
    // Exactly 1 pixel of the original image mapped to the 320px preview canvas
    const step = SIZE / s_base;

    if (e.key === "ArrowLeft") {
      panX = Math.max(-maxPanX, Math.min(maxPanX, panX - step));
      handled = true;
    } else if (e.key === "ArrowRight") {
      panX = Math.max(-maxPanX, Math.min(maxPanX, panX + step));
      handled = true;
    } else if (e.key === "ArrowUp") {
      panY = Math.max(-maxPanY, Math.min(maxPanY, panY - step));
      handled = true;
    } else if (e.key === "ArrowDown") {
      panY = Math.max(-maxPanY, Math.min(maxPanY, panY + step));
      handled = true;
    }

    if (handled) {
      e.preventDefault();
      debouncedSyncSettings();
    }
  }

  function handlePointerDown(e: PointerEvent) {
    if (!interactive) return;
    e.preventDefault();
    (e.currentTarget as HTMLElement).focus();
    isDragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    startX = e.clientX;
    startY = e.clientY;
    startPanX = panX;
    startPanY = panY;
  }

  function handlePointerMove(e: PointerEvent) {
    if (!interactive || !isDragging) return;
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;

    const newPanX = startPanX + dx;
    const newPanY = startPanY + dy;

    // Clamp limits
    panX = Math.max(-maxPanX, Math.min(maxPanX, newPanX));
    panY = Math.max(-maxPanY, Math.min(maxPanY, newPanY));

    debouncedSyncSettings();
  }

  function handlePointerUp(e: PointerEvent) {
    if (isDragging) {
      isDragging = false;
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
      clearTimeout(debounceTimeout);
      onchange?.(zoom, panX / SIZE, panY / SIZE);
    }
  }

  function handleWheel(e: WheelEvent) {
    if (!interactive) return;
    e.preventDefault();
    const delta = -e.deltaY * 0.002;
    const newZoom = Math.max(1.0, Math.min(10.0, zoom + delta));
    
    if (newZoom !== zoom) {
      zoom = newZoom;
      const limit = 160 * (zoom - 1);
      panX = Math.max(-limit, Math.min(limit, panX));
      panY = Math.max(-limit, Math.min(limit, panY));
      debouncedSyncSettings();
    }
  }

  function handleZoomSlider(e: Event) {
    const target = e.target as HTMLInputElement;
    zoom = parseFloat(target.value);
    const limit = 160 * (zoom - 1);
    panX = Math.max(-limit, Math.min(limit, panX));
    panY = Math.max(-limit, Math.min(limit, panY));
    debouncedSyncSettings();
  }

  function reset() {
    zoom = 1.0;
    panX = 0.0;
    panY = 0.0;
    clearTimeout(debounceTimeout);
    onchange?.(zoom, panX / SIZE, panY / SIZE);
  }

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

<div class="preview-wrapper">
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div 
    class="canvas-container"
    class:interactive={interactive}
    tabindex={interactive ? 0 : -1}
    onpointerdown={handlePointerDown}
    onpointermove={handlePointerMove}
    onpointerup={handlePointerUp}
    onpointercancel={handlePointerUp}
    onwheel={handleWheel}
    onkeydown={handleKeyDown}
    role="application"
    aria-label="Hexagon LED preview panel"
  >
    <canvas bind:this={canvas} width={SIZE} height={SIZE}></canvas>
    
    {#if isLoading}
      <div class="loading-overlay">
        <div class="spinner"></div>
      </div>
    {/if}

    {#if interactive}
      <div class="drag-hint">Drag grid to Pan • Scroll to Zoom</div>
    {/if}
  </div>

  {#if interactive}
    <div class="control-row">
      <div class="zoom-label">Zoom: {zoom.toFixed(1)}x</div>
      <input
        type="range"
        min="1.0"
        max="10.0"
        step="0.1"
        value={zoom}
        oninput={handleZoomSlider}
        class="zoom-slider"
      />
      <button class="btn-reset" onclick={reset} title="Reset Zoom/Pan">
        ↺
      </button>
    </div>
  {/if}
</div>

<style>
  .preview-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    width: 100%;
    margin: 0 auto;
  }

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
    position: relative;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(11, 11, 14, 0.6);
    backdrop-filter: blur(4px);
    border-radius: 20px;
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 20;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-radius: 50%;
    border-top-color: #007aff;
    border-right-color: #00c781;
    animation: spin 1s ease-in-out infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .canvas-container.interactive {
    cursor: grab;
    user-select: none;
    touch-action: none;
    position: relative;
  }

  .canvas-container.interactive:active {
    cursor: grabbing;
  }

  .canvas-container:focus {
    outline: none;
  }

  .canvas-container.interactive:focus,
  .canvas-container.interactive:focus-visible {
    border-color: #007aff;
    box-shadow: 0 0 0 2px rgba(0, 122, 255, 0.3), 0 10px 30px rgba(0, 0, 0, 0.3);
  }

  canvas {
    display: block;
    background: transparent;
  }

  .drag-hint {
    position: absolute;
    bottom: 25px;
    left: 50%;
    transform: translateX(-50%);
    background-color: rgba(11, 11, 14, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.05);
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.5);
    pointer-events: none;
    backdrop-filter: blur(4px);
    letter-spacing: 0.5px;
    white-space: nowrap;
  }

  .control-row {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 320px;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    padding: 10px 14px;
    border-radius: 12px;
    box-sizing: border-box;
  }

  .zoom-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: #8c8c9a;
    width: 75px;
    text-align: left;
  }

  .zoom-slider {
    flex: 1;
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.1);
    outline: none;
    cursor: pointer;
  }

  .zoom-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #007aff;
    transition: transform 0.1s;
  }

  .zoom-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
    background: #00c781;
  }

  .btn-reset {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.4);
    font-size: 1.1rem;
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: all 0.2s;
    width: 24px;
    height: 24px;
  }

  .btn-reset:hover {
    background-color: rgba(255, 255, 255, 0.05);
    color: #ffffff;
    transform: rotate(-45deg);
  }
</style>
