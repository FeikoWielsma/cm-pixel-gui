<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";

  let { path, kind, settings, onchange } = $props<{
    path: string;
    kind: "image" | "gif";
    settings: any;
    onchange: (zoom: number, panX: number, panY: number) => void;
  }>();

  // Retrieve stored parameters or default
  let initialParams = $derived(
    settings?.image_parameters?.[path] || { zoom: 1.0, pan_x: 0.0, pan_y: 0.0 }
  );

  // Local state representing the current pan and zoom
  let zoom = $state(1.0);
  let panX = $state(0.0);
  let panY = $state(0.0);

  // Sync from props if path or settings change
  $effect(() => {
    zoom = initialParams.zoom;
    // Normalized back to pixels on the 280px preview container
    panX = initialParams.pan_x * 280;
    panY = initialParams.pan_y * 280;
  });

  // Calculate clamp limits on the frontend based on the 280px size
  let maxPanX = $derived(140 * (zoom - 1));
  let maxPanY = $derived(140 * (zoom - 1));

  // Pointer state
  let isDragging = false;
  let startX = 0;
  let startY = 0;
  let startPanX = 0;
  let startPanY = 0;

  // Debouncing setup
  let debounceTimeout: any;

  function debouncedSyncSettings() {
    clearTimeout(debounceTimeout);
    debounceTimeout = setTimeout(() => {
      onchange(zoom, panX / 280, panY / 280);
    }, 50);
  }

  function handlePointerDown(e: PointerEvent) {
    e.preventDefault();
    isDragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    startX = e.clientX;
    startY = e.clientY;
    startPanX = panX;
    startPanY = panY;
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging) return;
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;

    const newPanX = startPanX + dx;
    const newPanY = startPanY + dy;

    // Clamp to prevent moving past image edges
    panX = Math.max(-maxPanX, Math.min(maxPanX, newPanX));
    panY = Math.max(-maxPanY, Math.min(maxPanY, newPanY));

    debouncedSyncSettings();
  }

  function handlePointerUp(e: PointerEvent) {
    if (isDragging) {
      isDragging = false;
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
      clearTimeout(debounceTimeout);
      onchange(zoom, panX / 280, panY / 280);
    }
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const delta = -e.deltaY * 0.002;
    const newZoom = Math.max(1.0, Math.min(10.0, zoom + delta));
    
    if (newZoom !== zoom) {
      zoom = newZoom;
      // Adjust pan coordinates to be clamped inside the new zoom bounds
      const limitX = 140 * (zoom - 1);
      const limitY = 140 * (zoom - 1);
      panX = Math.max(-limitX, Math.min(limitX, panX));
      panY = Math.max(-limitY, Math.min(limitY, panY));
      
      debouncedSyncSettings();
    }
  }

  function handleZoomSlider(e: Event) {
    const target = e.target as HTMLInputElement;
    zoom = parseFloat(target.value);
    
    // Adjust pan coordinates to new bounds
    const limitX = 140 * (zoom - 1);
    const limitY = 140 * (zoom - 1);
    panX = Math.max(-limitX, Math.min(limitX, panX));
    panY = Math.max(-limitY, Math.min(limitY, panY));
    
    debouncedSyncSettings();
  }

  function reset() {
    zoom = 1.0;
    panX = 0.0;
    panY = 0.0;
    clearTimeout(debounceTimeout);
    onchange(zoom, panX / 280, panY / 280);
  }
</script>

<div class="zoom-pan-container">
  <div class="zoom-pan-hex-border">
    <div 
      class="zoom-pan-hex-inner"
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      onpointercancel={handlePointerUp}
      onwheel={handleWheel}
      role="application"
      aria-label="Zoom and pan preview image"
    >
      <img
        src={convertFileSrc(path)}
        alt="Zoom and pan preview"
        style="transform: translate({panX}px, {panY}px) scale({zoom});"
        class="zoom-pan-image"
      />
      <div class="drag-hint">Drag to Pan • Scroll to Zoom</div>
    </div>
  </div>

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
</div>

<style>
  .zoom-pan-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    width: 100%;
    margin: 0 auto;
  }

  /* Large pointy-topped hexagon with CM gradient border */
  .zoom-pan-hex-border {
    position: relative;
    width: 284px;
    height: 284px;
    background: linear-gradient(135deg, #007aff, #00c781);
    clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.4);
    transition: transform 0.3s ease;
  }

  .zoom-pan-hex-border:hover {
    transform: scale(1.01);
  }

  .zoom-pan-hex-inner {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 280px;
    height: 280px;
    clip-path: inherit;
    background-color: #0b0b0e;
    overflow: hidden;
    cursor: grab;
    user-select: none;
    touch-action: none;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .zoom-pan-hex-inner:active {
    cursor: grabbing;
  }

  .zoom-pan-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    pointer-events: none;
    transition: transform 0.05s linear; /* extremely short transition for butter-smooth rendering */
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
  }

  .control-row {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 284px;
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
