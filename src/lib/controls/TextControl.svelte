<script lang="ts">
  let {
    textString = $bindable(),
    textColorHex = $bindable(),
    textScroll = $bindable(),
    onapply,
  } = $props<{
    textString: string;
    textColorHex: string;
    textScroll: boolean;
    onapply: () => void;
  }>();
</script>

<div class="form-group">
  <label class="control-label" for="text-string">Text Message</label>
  <input type="text" id="text-string" bind:value={textString} placeholder="ENTER TEXT" />
</div>

<div class="form-row">
  <div class="form-group">
    <label class="control-label" for="text-color">Color</label>
    <div class="color-picker-row">
      <input type="color" id="text-color" bind:value={textColorHex} />
      <span class="color-text">{textColorHex.toUpperCase()}</span>
    </div>
  </div>
  <div class="form-group checkbox-group">
    <label class="checkbox-label" for="text-scroll">
      <input type="checkbox" id="text-scroll" bind:checked={textScroll} />
      Scroll Text
    </label>
  </div>
</div>

<button class="btn btn-primary mt-4" onclick={onapply}>Apply Text</button>

<style>
  .form-group {
    margin-bottom: 20px;
  }
  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    margin-top: 20px;
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
  input[type="text"] {
    width: 100%;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #ffffff;
    padding: 12px 16px;
    border-radius: 10px;
    box-sizing: border-box;
    font-size: 0.95rem;
    font-family: inherit;
    transition: border-color 0.2s ease;
  }
  input[type="text"]:focus {
    outline: none;
    border-color: #007aff;
    background-color: rgba(255, 255, 255, 0.03);
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
  .btn {
    display: inline-flex;
    justify-content: center;
    align-items: center;
    font-family: inherit;
    font-size: 0.95rem;
    font-weight: 600;
    padding: 12px 24px;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.2s ease;
    border: none;
    width: 100%;
    box-sizing: border-box;
  }
  .btn-primary {
    background-color: #007aff;
    color: #ffffff;
    box-shadow: 0 4px 16px rgba(0, 122, 255, 0.3);
  }
  .btn-primary:hover {
    background-color: #0062cc;
  }
  .mt-4 {
    margin-top: 16px;
  }
</style>
