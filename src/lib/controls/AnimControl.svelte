<script lang="ts" module>
  let cachedPreviews: Record<string, number[][][][]> | null = null;
</script>

<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { AnimInfo } from "../api";
  import layoutData from "../../../reference/layout.json";

  let {
    animations,
    selectedAnimId = $bindable(),
    animSpeed = $bindable(),
    toggleMode = false,
    loopIds = $bindable([]),
    starfieldDensity = $bindable(0.04),
    starfieldTrailLen = $bindable(1.5),
    animPalette = $bindable("rainbow"),
    animAngle = $bindable(0),
    fireIntensity = $bindable(1.0),
    metaballCount = $bindable(4),
    metaballSize = $bindable(1.0),
    auroraSize = $bindable(1.0),
    twinkleCount = $bindable(45),
    pinwheelArms = $bindable(5),
    breatheColorHex = $bindable("#007aff"),
    rubikAngleX = $bindable(33),
    rubikAngleY = $bindable(45),
    rubikScale = $bindable(6.3),
    rubikPanX = $bindable(0.0),
    rubikPanY = $bindable(0.0),
    icecubeAngleX = $bindable(35.26),
    icecubeAngleY = $bindable(45.0),
    icecubeScale = $bindable(9.0),
    icecubePanX = $bindable(0.0),
    icecubePanY = $bindable(0.0),
    icecubeColorHex = $bindable("#8cddff"),
    bounceBallSize = $bindable(1.5),
    juliaZoom = $bindable(1.0),
    onchange,
  } = $props<{
    animations: AnimInfo[];
    selectedAnimId: string;
    animSpeed: number;
    toggleMode?: boolean;
    loopIds?: string[];
    starfieldDensity?: number;
    starfieldTrailLen?: number;
    animPalette?: string;
    animAngle?: number;
    fireIntensity?: number;
    metaballCount?: number;
    metaballSize?: number;
    auroraSize?: number;
    twinkleCount?: number;
    pinwheelArms?: number;
    breatheColorHex?: string;
    rubikAngleX?: number;
    rubikAngleY?: number;
    rubikScale?: number;
    rubikPanX?: number;
    rubikPanY?: number;
    icecubeAngleX?: number;
    icecubeAngleY?: number;
    icecubeScale?: number;
    icecubePanX?: number;
    icecubePanY?: number;
    icecubeColorHex?: string;
    bounceBallSize?: number;
    juliaZoom?: number;
    onchange: () => void;
  }>();

  const palettes = [
    { id: "fixed", label: "Fixed Color", gradient: "linear-gradient(135deg, #007aff, #00c781)" },
    { id: "rainbow", label: "Rainbow", gradient: "linear-gradient(90deg, #ff0000, #ff7f00, #ffff00, #00ff00, #0000ff, #4b0082, #8f00ff)" },
    { id: "magma", label: "Magma/Fire", gradient: "linear-gradient(90deg, #320000, #b41400, #ff6400, #ffd200, #ffffb4)" },
    { id: "ice", label: "Ice Blue", gradient: "linear-gradient(90deg, #001e64, #0064c8, #00c8ff, #64ffff, #c8ffff)" },
    { id: "ice_fire", label: "Ice & Fire", gradient: "linear-gradient(90deg, #001e78, #0064ff, #960096, #ff3200, #ffd264)" },
    { id: "toxic", label: "Toxic Green", gradient: "linear-gradient(90deg, #002800, #007814, #00dc32, #64ff50, #c8ffb4)" },
    { id: "synthwave", label: "Synthwave", gradient: "linear-gradient(90deg, #2b0080, #ff0080, #00f2fe, #ffe000)" },
    { id: "forest", label: "Forest", gradient: "linear-gradient(90deg, #143214, #286432, #64a050, #c8d278, #e6f0be)" },
    { id: "catppuccin", label: "Catppuccin", gradient: "linear-gradient(90deg, #c6a0f6, #8aadf4, #8bd5ca, #eed49f, #f5a97f, #ed8796)" },
    { id: "everforest", label: "Everforest", gradient: "linear-gradient(90deg, #a7c080, #dbbc7f, #e67e80, #7fbbb3, #d3c6aa)" },
    { id: "nord", label: "Nord", gradient: "linear-gradient(90deg, #8fbcbb, #88c0d0, #81a1c1, #5e81ac, #bf616a, #d8dee9)" },
    { id: "dracula", label: "Dracula", gradient: "linear-gradient(90deg, #bd93f9, #ff79c6, #8be9fd, #50fa7b, #ffb86c, #ff5555)" },
    { id: "sunset", label: "Sunset", gradient: "linear-gradient(90deg, #7209b7, #b5179e, #f72585, #fc4c02, #ffba08)" },
    { id: "cyberpunk", label: "Cyberpunk", gradient: "linear-gradient(90deg, #ffdc00, #ffb400, #00c8ff, #00fff0, #fff064)" },
  ];

  let showPalettePicker = $derived([
    "plasma", "rainbow", "swirl", "ripple", "tunnel",
    "metaballs", "fireworks", "pinwheel", "interference",
    "bounce", "twinkle", "rubik", "breathe",
    "torus", "julia", "chroma_life", "icecube", "fire"
  ].includes(selectedAnimId));

  const getThumbnailStyle = (id: string) => {
    switch (id) {
      case "plasma":
        return "background: radial-gradient(circle at 20% 20%, #ff007f, transparent 60%), radial-gradient(circle at 80% 80%, #00ffff, transparent 60%), radial-gradient(circle at 50% 50%, #7209b7, #08080a);";
      case "rainbow":
        return "background: linear-gradient(135deg, #ff0000, #ff7f00, #ffff00, #00ff00, #0000ff, #8f00ff);";
      case "swirl":
        return "background: conic-gradient(from 45deg, #ff00c7, #0088ff, #00c781, #ff00c7);";
      case "ripple":
        return "background: radial-gradient(circle, transparent 20%, #007aff 22%, transparent 32%, #007aff 34%, transparent 44%, #007aff 46%, transparent 56%); background-color: #08080a;";
      case "breathe":
        return "background: radial-gradient(circle, rgba(0, 122, 255, 0.8) 0%, rgba(0, 122, 255, 0.2) 40%, transparent 70%); background-color: #08080a;";
      case "sparkle":
        return "background: radial-gradient(circle at 30% 30%, #fff 2px, transparent 4px), radial-gradient(circle at 70% 60%, #fff 2px, transparent 4px); background-color: #0b0b0e;";
      case "fire":
        return "background: linear-gradient(0deg, #ff3b30 0%, #ff9500 60%, #ffcc00 100%);";
      case "starfield":
        return "background: radial-gradient(1px 1px at 15px 15px, #fff 100%, transparent), radial-gradient(1px 1px at 50px 25px, #fff 100%, transparent), radial-gradient(1.5px 1.5px at 70px 10px, #fff 100%, transparent), radial-gradient(1px 1px at 30px 45px, #fff 100%, transparent); background-color: #08080a;";
      case "tunnel":
        return "background: repeating-radial-gradient(circle, #08080a, #08080a 6px, #007aff 8px, #08080a 10px);";
      case "metaballs":
        return "background: radial-gradient(circle at 35% 50%, #ff00c7 16%, transparent 40%), radial-gradient(circle at 65% 50%, #0088ff 18%, transparent 42%); background-color: #0b0b0e;";
      case "matrix":
        return "background: linear-gradient(180deg, #00ff00 0%, transparent 80%); background-color: #08080a;";
      case "fireworks":
        return "background: radial-gradient(circle at 50% 50%, #ffcc00 3px, transparent 6px), radial-gradient(circle at 25% 35%, #ff3b30 2px, transparent 4px), radial-gradient(circle at 75% 65%, #00c781 2px, transparent 4px); background-color: #08080a;";
      case "aurora":
        return "background: linear-gradient(125deg, #00c781, #007aff, #7209b7, #08080a);";
      case "comet":
        return "background: linear-gradient(135deg, #00c781 0%, rgba(0, 199, 129, 0.2) 40%, transparent 85%); background-color: #08080a;";
      case "pinwheel":
        return "background: conic-gradient(#ff007f 0deg 45deg, #00ffff 45deg 90deg, #ffcc00 90deg 135deg, #00c781 135deg 180deg, #ff007f 180deg);";
      case "bounce":
        return "background: radial-gradient(circle at 50% 35%, #ff3b30 20%, transparent 24%); background-color: #0b0b0e;";
      case "interference":
        return "background: repeating-linear-gradient(45deg, rgba(0,122,255,0.15), rgba(0,122,255,0.15) 3px, transparent 3px, transparent 6px), repeating-linear-gradient(-45deg, rgba(0,199,129,0.15), rgba(0,199,129,0.15) 3px, transparent 3px, transparent 6px); background-color: #08080a;";
      case "twinkle":
        return "background: radial-gradient(circle at 20% 30%, #fff 1px, transparent 2px), radial-gradient(circle at 75% 20%, #fff 1.5px, transparent 3px), radial-gradient(circle at 40% 70%, #fff 1px, transparent 2px); background-color: #08080a;";
      case "rubik":
        return "background: #0b0b0e;";
      case "icecube":
        return "background: radial-gradient(circle at 30% 30%, #e0f7ff, transparent 80%), linear-gradient(135deg, #00b0ff, #0060b2);";
      case "torus":
        return "background: radial-gradient(circle at 50% 50%, transparent 18%, #bd93f9 22%, #8be9fd 30%, transparent 34%, transparent 42%, #bd93f9 44%, transparent 48%); background-color: #0b0b0e;";
      case "julia":
        return "background: radial-gradient(ellipse at 30% 40%, #ff79c6 0%, transparent 35%), radial-gradient(ellipse at 70% 60%, #8be9fd 0%, transparent 30%), radial-gradient(ellipse at 50% 50%, #bd93f9 0%, transparent 50%); background-color: #08080a;";
      case "chroma_life":
        return "background-color: #08080a; background-image: radial-gradient(circle at 25% 25%, #50fa7b 1.5px, transparent 2px), radial-gradient(circle at 75% 25%, #50fa7b 1.5px, transparent 2px), radial-gradient(circle at 50% 50%, #ffffff 1.5px, transparent 2px), radial-gradient(circle at 25% 75%, #50fa7b 1.5px, transparent 2px), radial-gradient(circle at 75% 75%, #50fa7b 1.5px, transparent 2px), radial-gradient(circle at 50% 25%, #ffffff 1.5px, transparent 2px); background-size: 100% 100%;";
      case "soccer":
        return "background: linear-gradient(90deg, #0a5c12 0%, #0d7018 25%, #0a5c12 50%, #0d7018 75%, #0a5c12 100%);";
      default:
        return "background: linear-gradient(135deg, #007aff, #00c781)";
    }
  };

  let containerWidth = $state(245);
  let columns = $derived(Math.max(3, Math.floor((containerWidth - 24 - 35) / 70)));

  let canvases: Record<string, HTMLCanvasElement> = {};
  let animationFrameId: number;
  let currentFrameIndex = 0;

  async function loadPreviewsAndPlay() {
    if (!cachedPreviews) {
      try {
        cachedPreviews = await invoke<Record<string, number[][][][]>>("get_anim_previews_data");
      } catch (e) {
        console.error("Failed to fetch anim previews data:", e);
        return;
      }
    }

    let lastTime = performance.now();
    const frameInterval = 1000 / 15; // 15 FPS

    function loop(time: number) {
      const elapsed = time - lastTime;
      if (elapsed >= frameInterval) {
        lastTime = time - (elapsed % frameInterval);

        currentFrameIndex = (currentFrameIndex + 1) % 9000;

        if (cachedPreviews) {
          for (const [id, frames] of Object.entries(cachedPreviews)) {
            const canvas = canvases[id];
            const frame = frames[currentFrameIndex % frames.length];
            if (canvas && frame) {
              drawMiniPreview(canvas, frame);
            }
          }
        }
      }
      animationFrameId = requestAnimationFrame(loop);
    }

    animationFrameId = requestAnimationFrame(loop);
  }

  function drawMiniPreview(canvas: HTMLCanvasElement, px: number[][][]) {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const size = canvas.width; // 132
    ctx.clearRect(0, 0, size, size);

    const cellSize = size / 32;
    const radius = cellSize * 0.35;

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
            ctx.fillStyle = "rgba(45, 45, 55, 0.4)";
          } else {
            ctx.fillStyle = `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`;
          }
          ctx.fill();
        }
      }
    }
  }

  onMount(() => {
    loadPreviewsAndPlay();
  });

  onDestroy(() => {
    cancelAnimationFrame(animationFrameId);
  });
</script>

<div class="form-group">
  <span class="control-label">{toggleMode ? "Select Effects to Include" : "Animation Effect"}</span>
  <div class="effect-list-wrapper" bind:clientWidth={containerWidth}>
    <div class="hex-grid" style="grid-template-columns: repeat({columns}, 70px); width: {columns * 70 + 35}px;">
      {#each animations as anim, i (anim.id)}
        {@const row = Math.floor(i / columns)}
        {@const isShifted = row % 2 === 1}
        {@const isActive = toggleMode ? loopIds.includes(anim.id) : selectedAnimId === anim.id}
        <div class="hex-wrapper" class:shifted={isShifted}>
          <div
            role="button"
            tabindex="0"
            class="hex-cell"
            class:active={isActive}
            class:toggle-mode={toggleMode}
            onclick={() => {
              if (toggleMode) {
                loopIds = loopIds.includes(anim.id)
                  ? loopIds.filter(id => id !== anim.id)
                  : [...loopIds, anim.id];
              } else {
                selectedAnimId = anim.id;
              }
              onchange();
            }}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                if (toggleMode) {
                  loopIds = loopIds.includes(anim.id)
                    ? loopIds.filter(id => id !== anim.id)
                    : [...loopIds, anim.id];
                } else {
                  selectedAnimId = anim.id;
                }
                onchange();
              }
            }}
            title={anim.label}
          >
            <div class="hex-inner">
              <div class="effect-preview">
                <canvas
                  bind:this={canvases[anim.id]}
                  width={132}
                  height={132}
                  class="hex-canvas"
                ></canvas>
              </div>
              {#if toggleMode && isActive}
                <div class="hex-check">✓</div>
              {/if}
              <div class="hex-label">
                <span class="hex-label-text">{anim.label}</span>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<div class="form-group mt-4">
  <div class="slider-header">
    <label class="control-label" for="anim-speed">Animation Speed</label>
    <span class="slider-val">{animSpeed.toFixed(1)}x</span>
  </div>
  <input
    type="range"
    id="anim-speed"
    min="0.1"
    max="3.0"
    step="0.1"
    bind:value={animSpeed}
    onchange={onchange}
  />
</div>

{#if selectedAnimId === 'starfield'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="starfield-density">Star Density</label>
      <span class="slider-val">{(starfieldDensity * 1000).toFixed(0)}</span>
    </div>
    <input
      type="range"
      id="starfield-density"
      min="0.005"
      max="0.15"
      step="0.005"
      bind:value={starfieldDensity}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="starfield-trail-len">Trail Length</label>
      <span class="slider-val">{starfieldTrailLen.toFixed(1)}x</span>
    </div>
    <input
      type="range"
      id="starfield-trail-len"
      min="0.1"
      max="6.0"
      step="0.1"
      bind:value={starfieldTrailLen}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'rainbow'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="rainbow-angle">Gradient Angle</label>
      <span class="slider-val">{animAngle.toFixed(0)}°</span>
    </div>
    <input
      type="range"
      id="rainbow-angle"
      min="0"
      max="360"
      step="1"
      bind:value={animAngle}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'fire'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="fire-intensity">Flame Intensity</label>
      <span class="slider-val">{fireIntensity.toFixed(1)}x</span>
    </div>
    <input
      type="range"
      id="fire-intensity"
      min="0.2"
      max="2.0"
      step="0.1"
      bind:value={fireIntensity}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'metaballs'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="metaball-count">Ball Count</label>
      <span class="slider-val">{metaballCount.toFixed(0)}</span>
    </div>
    <input
      type="range"
      id="metaball-count"
      min="2"
      max="8"
      step="1"
      bind:value={metaballCount}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="metaball-size">Ball Size</label>
      <span class="slider-val">{metaballSize.toFixed(1)}x</span>
    </div>
    <input
      type="range"
      id="metaball-size"
      min="0.3"
      max="2.5"
      step="0.1"
      bind:value={metaballSize}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'aurora'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="aurora-size">Band Thickness</label>
      <span class="slider-val">{auroraSize.toFixed(1)}x</span>
    </div>
    <input
      type="range"
      id="aurora-size"
      min="0.3"
      max="2.5"
      step="0.1"
      bind:value={auroraSize}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'pinwheel'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="pinwheel-arms">Branches</label>
      <span class="slider-val">{pinwheelArms.toFixed(0)}</span>
    </div>
    <input
      type="range"
      id="pinwheel-arms"
      min="2"
      max="10"
      step="1"
      bind:value={pinwheelArms}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'twinkle'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="twinkle-count">Star Count</label>
      <span class="slider-val">{twinkleCount.toFixed(0)}</span>
    </div>
    <input
      type="range"
      id="twinkle-count"
      min="10"
      max="120"
      step="5"
      bind:value={twinkleCount}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'rubik'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="rubik-angle-x">Tilt X</label>
      <span class="slider-val">{rubikAngleX.toFixed(0)}°</span>
    </div>
    <input
      type="range"
      id="rubik-angle-x"
      min="0"
      max="360"
      step="1"
      bind:value={rubikAngleX}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="rubik-angle-y">Tilt Y</label>
      <span class="slider-val">{rubikAngleY.toFixed(0)}°</span>
    </div>
    <input
      type="range"
      id="rubik-angle-y"
      min="0"
      max="360"
      step="1"
      bind:value={rubikAngleY}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="rubik-scale">Zoom</label>
      <span class="slider-val">{rubikScale.toFixed(1)}x</span>
    </div>
    <input
      type="range"
      id="rubik-scale"
      min="1.0"
      max="15.0"
      step="0.1"
      bind:value={rubikScale}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="rubik-pan-x">Pan X</label>
      <span class="slider-val">{rubikPanX.toFixed(1)}</span>
    </div>
    <input
      type="range"
      id="rubik-pan-x"
      min="-15.0"
      max="15.0"
      step="0.5"
      bind:value={rubikPanX}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="rubik-pan-y">Pan Y</label>
      <span class="slider-val">{rubikPanY.toFixed(1)}</span>
    </div>
    <input
      type="range"
      id="rubik-pan-y"
      min="-15.0"
      max="15.0"
      step="0.5"
      bind:value={rubikPanY}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'icecube'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="icecube-angle-x">Tilt X</label>
      <span class="slider-val">{icecubeAngleX.toFixed(0)}°</span>
    </div>
    <input
      type="range"
      id="icecube-angle-x"
      min="0"
      max="360"
      step="1"
      bind:value={icecubeAngleX}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="icecube-angle-y">Tilt Y</label>
      <span class="slider-val">{icecubeAngleY.toFixed(0)}°</span>
    </div>
    <input
      type="range"
      id="icecube-angle-y"
      min="0"
      max="360"
      step="1"
      bind:value={icecubeAngleY}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="icecube-scale">Zoom</label>
      <span class="slider-val">{icecubeScale.toFixed(1)}x</span>
    </div>
    <input
      type="range"
      id="icecube-scale"
      min="1.0"
      max="15.0"
      step="0.1"
      bind:value={icecubeScale}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="icecube-pan-x">Pan X</label>
      <span class="slider-val">{icecubePanX.toFixed(1)}</span>
    </div>
    <input
      type="range"
      id="icecube-pan-x"
      min="-15.0"
      max="15.0"
      step="0.5"
      bind:value={icecubePanX}
      onchange={onchange}
    />
  </div>

  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="icecube-pan-y">Pan Y</label>
      <span class="slider-val">{icecubePanY.toFixed(1)}</span>
    </div>
    <input
      type="range"
      id="icecube-pan-y"
      min="-15.0"
      max="15.0"
      step="0.5"
      bind:value={icecubePanY}
      onchange={onchange}
    />
  </div>
{/if}


{#if selectedAnimId === 'bounce'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="bounce-ball-size">Ball Size</label>
      <span class="slider-val">{bounceBallSize.toFixed(1)}</span>
    </div>
    <input
      type="range"
      id="bounce-ball-size"
      min="0.5"
      max="6.0"
      step="0.1"
      bind:value={bounceBallSize}
      onchange={onchange}
    />
  </div>
{/if}

{#if selectedAnimId === 'julia'}
  <div class="form-group mt-4 animate-fade-in">
    <div class="slider-header">
      <label class="control-label" for="julia-zoom">Zoom</label>
      <span class="slider-val">{juliaZoom.toFixed(1)}x</span>
    </div>
    <input
      type="range"
      id="julia-zoom"
      min="0.3"
      max="3.0"
      step="0.1"
      bind:value={juliaZoom}
      onchange={onchange}
    />
  </div>
{/if}

{#if showPalettePicker}
  <div class="form-group mt-4 animate-fade-in">
    <span class="control-label">Color Palette</span>
    <div class="palette-grid">
      {#each palettes as pal}
        <button
          type="button"
          class="palette-compact-btn"
          class:active={animPalette === pal.id}
          onclick={() => { animPalette = pal.id; onchange(); }}
          title={pal.label}
        >
          <div class="palette-swatch" style="background: {pal.gradient}"></div>
          <span class="palette-compact-label">{pal.label}</span>
        </button>
      {/each}
    </div>
  </div>

  {#if animPalette === 'fixed'}
    <div class="form-group mt-4 animate-fade-in">
      <span class="control-label">Fixed Color</span>
      <div class="color-picker-row">
        {#if selectedAnimId === 'icecube'}
          <input type="color" id="icecube-color" bind:value={icecubeColorHex} oninput={onchange} />
          <span class="color-text">{icecubeColorHex.toUpperCase()}</span>
        {:else}
          <input type="color" id="breathe-color" bind:value={breatheColorHex} oninput={onchange} />
          <span class="color-text">{breatheColorHex.toUpperCase()}</span>
        {/if}
      </div>
    </div>
  {/if}
{/if}

<style>
  .form-group {
    margin-bottom: 20px;
  }
  .control-label {
    display: block;
    font-size: 0.85rem;
    font-weight: 600;
    color: #8c8c9a;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
  }
  .slider-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .slider-header .control-label {
    margin-bottom: 0;
  }
  .slider-val {
    font-size: 0.85rem;
    font-weight: 700;
    color: #007aff;
    background-color: rgba(0, 122, 255, 0.1);
    padding: 2px 8px;
    border-radius: 12px;
  }
  input[type="range"] {
    width: 100%;
    margin: 8px 0;
    background: rgba(255, 255, 255, 0.05);
    height: 6px;
    border-radius: 3px;
    outline: none;
    appearance: none;
  }
  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #007aff;
    cursor: pointer;
    transition: transform 0.1s ease;
    box-shadow: 0 0 10px rgba(0, 122, 255, 0.5);
  }
  input[type="range"]::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }
  .mt-4 {
    margin-top: 16px;
  }
  .animate-fade-in {
    animation: fadeIn 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .palette-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
    margin-top: 8px;
  }
  .palette-compact-btn {
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    padding: 6px 8px;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
    overflow: hidden;
  }
  .palette-compact-btn:hover {
    background-color: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.12);
  }
  .palette-compact-btn.active {
    background-color: rgba(0, 122, 255, 0.08);
    border-color: #007aff;
    box-shadow: 0 0 8px rgba(0, 122, 255, 0.2);
  }
  .palette-swatch {
    width: 24px;
    height: 12px;
    border-radius: 4px;
    flex-shrink: 0;
  }
  .palette-compact-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: #8c8c9a;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: color 0.15s ease;
  }
  .palette-compact-btn.active .palette-compact-label {
    color: #ffffff;
  }
  .color-picker-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
  }
  input[type="color"] {
    border: none;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    cursor: pointer;
    background: transparent;
    padding: 0;
    overflow: hidden;
  }
  input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
  }
  input[type="color"]::-webkit-color-swatch {
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 50%;
  }
  .color-text {
    font-family: monospace;
    font-size: 0.95rem;
    color: #e5e5ed;
  }

  .effect-list-wrapper {
    max-height: 280px;
    overflow-y: auto;
    width: 100%;
    padding-right: 4px;
  }
  .effect-list-wrapper::-webkit-scrollbar {
    width: 6px;
  }
  .effect-list-wrapper::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.01);
    border-radius: 3px;
  }
  .effect-list-wrapper::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }
  .effect-list-wrapper::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  /* Hex grid layout styles */
  .hex-grid {
    display: grid;
    grid-auto-rows: 60px;
    column-gap: 0;
    row-gap: 0;
    justify-content: center;
    padding-top: 15px;
    padding-bottom: 35px;
    margin: 0 auto;
  }

  .hex-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    position: relative;
    width: 70px;
    height: 80px;
  }

  /* Shift alternate rows horizontally */
  .hex-wrapper.shifted {
    transform: translateX(35px);
  }

  .hex-cell {
    position: relative;
    width: 70px;
    height: 80px;
    background-color: rgba(255, 255, 255, 0.12); /* slight borders */
    clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
    cursor: pointer;
    transition: transform 0.2s, filter 0.2s, background-color 0.2s;
    overflow: hidden;
    box-sizing: border-box;
    border: none;
    padding: 0;
    margin: 0;
  }

  .hex-cell:hover {
    background-color: rgba(255, 255, 255, 0.35); /* hover border highlight */
    transform: scale(1.05);
    z-index: 10;
  }

  .hex-cell.active {
    background: linear-gradient(135deg, #007aff, #00c781); /* gradient active border */
    filter: drop-shadow(0 0 6px rgba(0, 122, 255, 0.8));
    transform: scale(1.05);
    z-index: 10;
  }

  /* In toggle mode, inactive cells are slightly dimmed */
  .hex-cell.toggle-mode:not(.active) {
    opacity: 0.55;
  }
  .hex-cell.toggle-mode:not(.active):hover {
    opacity: 0.85;
  }

  .hex-check {
    position: absolute;
    top: 12%;
    right: 20%;
    font-size: 0.75rem;
    font-weight: 900;
    color: #00c781;
    text-shadow: 0 0 6px rgba(0, 199, 129, 0.8);
    pointer-events: none;
    z-index: 6;
    line-height: 1;
  }

  .hex-inner {
    position: absolute;
    top: 2px;
    left: 2px;
    width: calc(100% - 4px);
    height: calc(100% - 4px);
    clip-path: inherit;
    background-color: #0b0b0e;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .effect-preview {
    width: 100%;
    height: 100%;
    position: absolute;
    top: 0;
    left: 0;
    overflow: hidden;
  }

  .hex-label {
    position: absolute;
    top: 55%;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(to bottom, transparent 0%, rgba(0, 0, 0, 0.8) 30%, rgba(0, 0, 0, 0.92) 70%);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    padding-top: 3px;
    pointer-events: none;
    z-index: 5;
    box-sizing: border-box;
  }

  .hex-label-text {
    font-size: 0.55rem;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.9);
    text-transform: uppercase;
    letter-spacing: 0.2px;
    white-space: normal;
    word-break: break-word;
    text-align: center;
    line-height: 1.25;
    width: 90%;
  }

  .hex-cell.active .hex-label-text {
    color: #ffffff;
  }

  .hex-canvas {
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
