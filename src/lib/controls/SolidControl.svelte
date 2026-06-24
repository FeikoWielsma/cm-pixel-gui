<script lang="ts">
  let { solidColorHex = $bindable(), onchange } = $props<{
    solidColorHex: string;
    onchange: () => void;
  }>();
</script>

<div class="form-group">
  <label class="control-label" for="solid-color">Fill Color</label>
  <div class="color-picker-row">
    <input type="color" id="solid-color" bind:value={solidColorHex} oninput={onchange} />
    <span class="color-text">{solidColorHex.toUpperCase()}</span>
  </div>
</div>

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
  .color-picker-row {
    display: flex;
    align-items: center;
    gap: 12px;
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
</style>
