<script lang="ts">
  import type { AnimInfo } from "../api";

  let {
    animations,
    selectedAnimId = $bindable(),
    animSpeed = $bindable(),
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
    bounceBallSize = $bindable(1.5),
    juliaZoom = $bindable(1.0),
    onchange,
  } = $props<{
    animations: AnimInfo[];
    selectedAnimId: string;
    animSpeed: number;
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
    bounceBallSize?: number;
    juliaZoom?: number;
    onchange: () => void;
  }>();

  const palettes = [
    { id: "fixed", label: "Fixed Color", gradient: "linear-gradient(135deg, #007aff, #00c781)" },
    { id: "rainbow", label: "Rainbow", gradient: "linear-gradient(90deg, #ff0000, #ff7f00, #ffff00, #00ff00, #0000ff, #4b0082, #8f00ff)" },
    { id: "catppuccin", label: "Catppuccin", gradient: "linear-gradient(90deg, #c6a0f6, #8aadf4, #8bd5ca, #eed49f, #f5a97f, #ed8796)" },
    { id: "everforest", label: "Everforest", gradient: "linear-gradient(90deg, #a7c080, #dbbc7f, #e67e80, #7fbbb3, #d3c6aa)" },
    { id: "nord", label: "Nord", gradient: "linear-gradient(90deg, #8fbcbb, #88c0d0, #81a1c1, #5e81ac, #bf616a, #d8dee9)" },
    { id: "dracula", label: "Dracula", gradient: "linear-gradient(90deg, #bd93f9, #ff79c6, #8be9fd, #50fa7b, #ffb86c, #ff5555)" },
    { id: "sunset", label: "Sunset", gradient: "linear-gradient(90deg, #7209b7, #b5179e, #f72585, #fc4c02, #ffba08)" },
    { id: "cyberpunk", label: "Cyberpunk", gradient: "linear-gradient(90deg, #ff007f, #8000ff, #00ffff, #ffff00)" },
  ];

  let showPalettePicker = $derived([
    "plasma", "rainbow", "swirl", "ripple", "tunnel",
    "metaballs", "fireworks", "pinwheel", "interference",
    "bounce", "twinkle", "rubik", "breathe",
    "torus", "julia", "chroma_life"
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
      default:
        return "background: linear-gradient(135deg, #007aff, #00c781);";
    }
  };
</script>

<div class="form-group">
  <span class="control-label">Animation Effect</span>
  <div class="effect-grid">
    {#each animations as anim}
      <button
        type="button"
        class="effect-btn"
        class:active={selectedAnimId === anim.id}
        onclick={() => { selectedAnimId = anim.id; onchange(); }}
      >
        <div class="effect-preview" style={getThumbnailStyle(anim.id)}>
          {#if anim.id === 'rubik'}
            <div class="rubik-grid">
              <div style="background: #ff3b30"></div>
              <div style="background: #4cd964"></div>
              <div style="background: #007aff"></div>
              <div style="background: #ffcc00"></div>
              <div style="background: #ff3b30"></div>
              <div style="background: #ffcc00"></div>
              <div style="background: #4cd964"></div>
              <div style="background: #007aff"></div>
              <div style="background: #ff3b30"></div>
            </div>
          {:else if anim.id === 'matrix'}
            <div class="matrix-preview-text">1 0 1 1 0</div>
          {:else if anim.id === 'breathe'}
            <div class="pulse-circle"></div>
          {/if}
        </div>
        <span class="effect-label">{anim.label}</span>
      </button>
    {/each}
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
          class="palette-btn"
          class:active={animPalette === pal.id}
          onclick={() => { animPalette = pal.id; onchange(); }}
        >
          <div class="palette-preview" style="background: {pal.gradient}"></div>
          <span class="palette-label">{pal.label}</span>
        </button>
      {/each}
    </div>
  </div>

  {#if animPalette === 'fixed'}
    <div class="form-group mt-4 animate-fade-in">
      <span class="control-label">Fixed Color</span>
      <div class="color-picker-row">
        <input type="color" id="breathe-color" bind:value={breatheColorHex} oninput={onchange} />
        <span class="color-text">{breatheColorHex.toUpperCase()}</span>
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
    gap: 8px;
    margin-top: 8px;
  }
  .palette-btn {
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
  }
  .palette-btn:hover {
    background-color: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.12);
  }
  .palette-btn.active {
    background-color: rgba(0, 122, 255, 0.08);
    border-color: #007aff;
    box-shadow: 0 0 12px rgba(0, 122, 255, 0.2);
  }
  .palette-preview {
    height: 12px;
    border-radius: 6px;
    margin-bottom: 6px;
    width: 100%;
  }
  .palette-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: #8c8c9a;
    transition: color 0.2s ease;
  }
  .palette-btn.active .palette-label {
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

  .effect-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
    gap: 12px;
    margin-top: 8px;
    max-height: 280px;
    overflow-y: auto;
    padding-right: 4px;
  }
  .effect-grid::-webkit-scrollbar {
    width: 6px;
  }
  .effect-grid::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.01);
    border-radius: 3px;
  }
  .effect-grid::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }
  .effect-grid::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }
  .effect-btn {
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }
  .effect-btn:hover {
    background-color: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.12);
    transform: translateY(-2px);
  }
  .effect-btn.active {
    background-color: rgba(0, 122, 255, 0.08);
    border-color: #007aff;
    box-shadow: 0 0 12px rgba(0, 122, 255, 0.2);
  }
  .effect-preview {
    width: 100%;
    height: 48px;
    border-radius: 6px;
    margin-bottom: 6px;
    position: relative;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .effect-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: #8c8c9a;
    transition: color 0.2s ease;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    width: 100%;
  }
  .effect-btn.active .effect-label {
    color: #ffffff;
  }
  .rubik-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1px;
    width: 24px;
    height: 24px;
    padding: 2px;
    background: #000;
    border-radius: 3px;
  }
  .matrix-preview-text {
    font-family: monospace;
    font-size: 0.55rem;
    color: #00ff00;
    text-shadow: 0 0 2px #00ff00;
    letter-spacing: 1px;
  }
  .pulse-circle {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #007aff;
    box-shadow: 0 0 8px #007aff;
    animation: previewPulse 2s infinite ease-in-out;
  }
  @keyframes previewPulse {
    0%, 100% { transform: scale(0.7); opacity: 0.5; }
    50% { transform: scale(1.1); opacity: 1; }
  }
</style>
