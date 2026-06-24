<script lang="ts">
  let {
    brightness = $bindable(),
    rotation = $bindable(),
    fps = $bindable(),
    autostart = $bindable(),
    onbrightness,
    onrotation,
    onfps,
    onautostart,
  } = $props<{
    brightness: number;
    rotation: number;
    fps: number;
    autostart: boolean;
    onbrightness: (e: Event) => void;
    onrotation: (e: Event) => void;
    onfps: (e: Event) => void;
    onautostart: () => void;
  }>();
</script>

<div class="settings-list card">
  <!-- Brightness -->
  <div class="form-group">
    <div class="slider-header">
      <label class="control-label" for="brightness">Display Brightness</label>
      <span class="slider-val">{brightness}%</span>
    </div>
    <input
      type="range"
      id="brightness"
      min="0"
      max="100"
      bind:value={brightness}
      oninput={onbrightness}
    />
  </div>

  <!-- Rotation -->
  <div class="form-group">
    <label class="control-label" for="rotation">Orientation Rotation</label>
    <div class="select-wrapper">
      <select id="rotation" bind:value={rotation} onchange={onrotation}>
        <option value={0}>Normal (0°)</option>
        <option value={90}>Rotate CW (90°)</option>
        <option value={180}>Upside Down (180°)</option>
        <option value={270}>Rotate CCW (270°)</option>
      </select>
    </div>
  </div>

  <!-- FPS -->
  <div class="form-group">
    <label class="control-label" for="fps">Target Frame Rate (FPS)</label>
    <div class="select-wrapper">
      <select id="fps" bind:value={fps} onchange={onfps}>
        <option value={5}>5 FPS (Low resource)</option>
        <option value={10}>10 FPS</option>
        <option value={15}>15 FPS (Default)</option>
        <option value={20}>20 FPS</option>
        <option value={30}>30 FPS (Smooth)</option>
        <option value={60}>60 FPS (Peak)</option>
      </select>
    </div>
  </div>

  <!-- Run at Startup -->
  <div class="form-group checkbox-group border-t">
    <label class="checkbox-label" for="autostart">
      <input
        type="checkbox"
        id="autostart"
        bind:checked={autostart}
        onchange={onautostart}
      />
      Start App Automatically on Boot
    </label>
  </div>
</div>

<style>
  .card {
    background-color: #0b0b0e;
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 16px;
    padding: 24px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }
  .form-group {
    margin-bottom: 20px;
  }
  .form-group:last-child {
    margin-bottom: 0;
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
  .select-wrapper {
    position: relative;
  }
  select {
    width: 100%;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #ffffff;
    padding: 12px 16px;
    border-radius: 10px;
    font-size: 0.95rem;
    font-family: inherit;
    cursor: pointer;
    appearance: none;
  }
  select option {
    background-color: #0b0b0e;
    color: #ffffff;
  }
  select:focus {
    outline: none;
    border-color: #007aff;
  }
  .select-wrapper::after {
    content: "↓";
    font-size: 0.8rem;
    color: #8c8c9a;
    position: absolute;
    right: 16px;
    top: 50%;
    transform: translateY(-50%);
    pointer-events: none;
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
  .checkbox-group {
    display: flex;
    align-items: center;
  }
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 0.95rem;
    color: #e5e5ed;
    cursor: pointer;
    user-select: none;
  }
  input[type="checkbox"] {
    appearance: none;
    width: 20px;
    height: 20px;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    cursor: pointer;
    display: grid;
    place-content: center;
  }
  input[type="checkbox"]::before {
    content: "✓";
    font-size: 0.75rem;
    color: transparent;
    font-weight: bold;
    transition: color 0.1s ease;
  }
  input[type="checkbox"]:checked {
    background-color: #007aff;
    border-color: #007aff;
  }
  input[type="checkbox"]:checked::before {
    color: #ffffff;
  }
  .border-t {
    border-top: 1px solid rgba(255, 255, 255, 0.04);
    padding-top: 20px;
  }
</style>
