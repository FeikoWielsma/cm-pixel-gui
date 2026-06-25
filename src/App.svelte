<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import {
    getStatus,
    getSettings,
    setMode,
    setBrightness,
    setRotation,
    setFps,
    setAutostart,
    setMinimizeToTray,
    setStartMinimized,
    listAnimations,
    pickFile,
    stopCmService,
    setRawFrame, // ponytail: import setRawFrame
    type Status,
    type Settings,
    type AnimInfo,
    type Rgb
  } from "./lib/api";
  import HexPreview from "./lib/HexPreview.svelte";
  import { DOOM } from "wasm-doom"; // ponytail: import DOOM class from wasm-doom

  // Child components
  import SolidControl from "./lib/controls/SolidControl.svelte";
  import TextControl from "./lib/controls/TextControl.svelte";
  import MediaControl from "./lib/controls/MediaControl.svelte";
  import AnimControl from "./lib/controls/AnimControl.svelte";
  import SettingsControl from "./lib/controls/SettingsControl.svelte";

  // Reactive state using Svelte 5 Runes
  let status: Status | null = $state(null);
  let settings: Settings | null = $state(null);
  let animations: AnimInfo[] = $state([]);
  let activeTab: "modes" | "settings" = $state("modes");

  // Input states for modes
  let selectedModeKind: string = $state("off");
  let solidColorHex: string = $state("#007aff");
  let textString: string = $state("HELLO WORLD");
  let textColorHex: string = $state("#ffffff");
  let textScroll: boolean = $state(true);
  let selectedImage: string = $state("");
  let selectedGif: string = $state("");
  let selectedAnimId: string = $state("plasma");
  let animSpeed: number = $state(1.0);
  let starfieldDensity: number = $state(0.04);
  let starfieldTrailLen: number = $state(1.5);
  let animPalette: string = $state("rainbow");
  let animAngle: number = $state(0);
  let fireIntensity: number = $state(1.0);
  let metaballCount: number = $state(4);
  let metaballSize: number = $state(1.0);
  let auroraSize: number = $state(1.0);
  let twinkleCount: number = $state(45);
  let pinwheelArms: number = $state(5);
  let breatheColorHex: string = $state("#007aff");
  let rubikAngleX: number = $state(33);
  let rubikAngleY: number = $state(45);
  let rubikScale: number = $state(6.3);
  let rubikPanX: number = $state(0.0);
  let rubikPanY: number = $state(0.0);
  let bounceBallSize: number = $state(1.5);
  let juliaZoom: number = $state(1.0);
  let icecubeAngleX: number = $state(35.26);
  let icecubeAngleY: number = $state(45.0);
  let icecubeScale: number = $state(9.0);
  let icecubePanX: number = $state(0.0);
  let icecubePanY: number = $state(0.0);
  let icecubeColorHex: string = $state("#8cddff");


  // Input states for device settings
  let brightness: number = $state(100);
  let rotation: number = $state(0);
  let fps: number = $state(15);
  let autostart: boolean = $state(false);
  let minimizeToTray: boolean = $state(true);
  let startMinimized: boolean = $state(false);

  // Conflict status helper
  let resolvingConflict: boolean = $state(false);

  // ponytail: DOOM game state & runner
  let isDoomActive = $state(false);
  let doomStarted = false;
  let gameInstance: any = null;

  // Loop mode states
  let loopInterval: number = $state(5);
  let loopEffectIds: string[] = $state([]);

  async function startDoom() {
    isDoomActive = true;
    if (doomStarted) {
      return;
    }
    setTimeout(async () => {
      const canvas = document.getElementById("doom-canvas") as HTMLCanvasElement;
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      const tempCanvas = document.createElement("canvas");
      tempCanvas.width = 32;
      tempCanvas.height = 32;
      const tempCtx = tempCanvas.getContext("2d");

      let lastSentTime = 0;
      doomStarted = true;

      gameInstance = new DOOM({
        screenWidth: 640,
        screenHeight: 400,
        enableLogs: false,
        onFrameRender: async ({ screen }) => {
          if (!isDoomActive || selectedModeKind !== "raw") return;

          // Render to browser canvas
          const frame = new ImageData(screen, 640, 400);
          ctx.putImageData(frame, 0, 0);

          // Downscale to 32x32 for CM Pixel Screen
          if (tempCtx) {
            tempCtx.drawImage(canvas, 0, 0, 32, 32);
            const imgData = tempCtx.getImageData(0, 0, 32, 32);

            // Throttling: send frame to device at target FPS (e.g. 15 FPS)
            const now = performance.now();
            const targetInterval = 1000 / (fps || 15);
            if (now - lastSentTime >= targetInterval) {
              lastSentTime = now;

              const px: Rgb[] = [];
              for (let i = 0; i < 1024; i++) {
                px.push([
                  imgData.data[i * 4],
                  imgData.data[i * 4 + 1],
                  imgData.data[i * 4 + 2],
                ]);
              }
              try {
                await setRawFrame(px);
              } catch (e) {
                console.error("Failed to send Doom frame to device", e);
              }
            }
          }
        },
      });

      await gameInstance.start();
    }, 100);
  }

  onMount(async () => {
    try {
      status = await getStatus();
    } catch (e) {
      console.error("Failed to get status:", e);
    }

    try {
      animations = await listAnimations();
      loopEffectIds = animations.map(a => a.id);
    } catch (e) {
      console.error("Failed to list animations:", e);
    }

    try {
      settings = await getSettings();
      if (settings) {
        syncSettingsToInputs(settings);
      }
    } catch (e) {
      console.error("Failed to get settings:", e);
    }

    let unlisten: (() => void) | undefined;
    try {
      unlisten = await listen<Status>("status", ({ payload }) => {
        status = payload;
        // Sync settings when mode is modified from elsewhere (e.g. tray menu)
        if (status && settings) {
          settings.mode = status.mode;
          syncSettingsToInputs(settings);
        }
      });
    } catch (e) {
      console.error("Failed to listen to status:", e);
    }

    return () => {
      unlisten?.();
    };
  });

  function syncSettingsToInputs(sets: Settings) {
    brightness = sets.brightness;
    rotation = sets.rotation;
    fps = sets.fps;
    autostart = sets.autostart;
    minimizeToTray = sets.minimize_to_tray;
    startMinimized = sets.start_minimized;

    const m = sets.mode;
    selectedModeKind = m.kind;
    if (m.kind === "solid") {
      solidColorHex = rgbToHex(m.rgb);
    } else if (m.kind === "text") {
      textString = m.text;
      textColorHex = rgbToHex(m.rgb);
      textScroll = m.scroll;
    } else if (m.kind === "image") {
      selectedImage = m.path;
    } else if (m.kind === "gif") {
      selectedGif = m.path;
    } else if (m.kind === "raw") {
      // ponytail: start Doom on load if raw mode is saved
      startDoom();
    } else if (m.kind === "loop") {
      loopEffectIds = m.ids || [];
      loopInterval = m.interval_secs || 5;
    } else if (m.kind === "anim") {
      selectedAnimId = m.id;
      animSpeed = m.speed;
      if (m.params) {
        if (typeof m.params.density === "number") starfieldDensity = m.params.density;
        if (typeof m.params.trail_len === "number") starfieldTrailLen = m.params.trail_len;
        if (typeof m.params.palette === "string") animPalette = m.params.palette;
        if (typeof m.params.angle === "number") animAngle = m.params.angle;
        if (typeof m.params.intensity === "number") fireIntensity = m.params.intensity;
        if (typeof m.params.count === "number") {
          if (m.id === "metaballs") metaballCount = m.params.count;
          else if (m.id === "twinkle") twinkleCount = m.params.count;
          else if (m.id === "pinwheel") pinwheelArms = m.params.count;
        }
        if (typeof m.params.ball_size === "number") {
          if (m.id === "metaballs") metaballSize = m.params.ball_size;
          else if (m.id === "bounce") bounceBallSize = m.params.ball_size;
        }
        if (typeof m.params.size === "number") auroraSize = m.params.size;
        if (typeof m.params.zoom === "number") juliaZoom = m.params.zoom;
        if (m.params.color && Array.isArray(m.params.color) && m.params.color.length >= 3) {
          breatheColorHex = rgbToHex(m.params.color as Rgb);
        }
        if (typeof m.params.angle_x === "number") rubikAngleX = m.params.angle_x;
        if (typeof m.params.angle_y === "number") rubikAngleY = m.params.angle_y;
        if (typeof m.params.scale === "number") rubikScale = m.params.scale;
        if (typeof m.params.pan_x === "number") rubikPanX = m.params.pan_x;
        if (typeof m.params.pan_y === "number") rubikPanY = m.params.pan_y;

        if (m.id === "icecube") {
          if (typeof m.params.angle_x === "number") icecubeAngleX = m.params.angle_x;
          if (typeof m.params.angle_y === "number") icecubeAngleY = m.params.angle_y;
          if (typeof m.params.scale === "number") icecubeScale = m.params.scale;
          if (typeof m.params.pan_x === "number") icecubePanX = m.params.pan_x;
          if (typeof m.params.pan_y === "number") icecubePanY = m.params.pan_y;
          if (m.params.color && Array.isArray(m.params.color) && m.params.color.length >= 3) {
            icecubeColorHex = rgbToHex(m.params.color as Rgb);
          }
        }

      }
    }
  }

  // Helper converters
  function hexToRgb(hex: string): Rgb {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return [r, g, b];
  }

  function rgbToHex(rgb: Rgb): string {
    const r = rgb[0].toString(16).padStart(2, "0");
    const g = rgb[1].toString(16).padStart(2, "0");
    const b = rgb[2].toString(16).padStart(2, "0");
    return `#${r}${g}${b}`;
  }

  // Triggering updates
  async function applyMode() {
    let modePayload;
    if (selectedModeKind === "off") {
      modePayload = { kind: "off" as const };
    } else if (selectedModeKind === "solid") {
      modePayload = { kind: "solid" as const, rgb: hexToRgb(solidColorHex) };
    } else if (selectedModeKind === "text") {
      modePayload = {
        kind: "text" as const,
        text: textString,
        rgb: hexToRgb(textColorHex),
        scroll: textScroll,
      };
    } else if (selectedModeKind === "image") {
      if (!selectedImage) return alert("Please select an image file first");
      modePayload = { kind: "image" as const, path: selectedImage };
    } else if (selectedModeKind === "gif") {
      if (!selectedGif) return alert("Please select a GIF file first");
      modePayload = { kind: "gif" as const, path: selectedGif };
    } else if (selectedModeKind === "anim") {
      let params: Record<string, any> | null = null;
      if (selectedAnimId === "starfield") {
        params = {
          density: starfieldDensity,
          trail_len: starfieldTrailLen,
        };
      } else if (
        selectedAnimId === "plasma" ||
        selectedAnimId === "swirl" ||
        selectedAnimId === "ripple" ||
        selectedAnimId === "tunnel" ||
        selectedAnimId === "fireworks" ||
        selectedAnimId === "interference" ||
        selectedAnimId === "torus" ||
        selectedAnimId === "chroma_life"
      ) {
        params = {
          palette: animPalette,
        };
      } else if (selectedAnimId === "julia") {
        params = {
          palette: animPalette,
          zoom: juliaZoom,
        };
      } else if (selectedAnimId === "bounce") {
        params = {
          palette: animPalette,
          ball_size: bounceBallSize,
        };
      } else if (selectedAnimId === "rubik") {
        params = {
          palette: animPalette,
          angle_x: rubikAngleX,
          angle_y: rubikAngleY,
          scale: rubikScale,
          pan_x: rubikPanX,
          pan_y: rubikPanY,
        };
      } else if (selectedAnimId === "icecube") {
        params = {
          palette: animPalette,
          angle_x: icecubeAngleX,
          angle_y: icecubeAngleY,
          scale: icecubeScale,
          pan_x: icecubePanX,
          pan_y: icecubePanY,
          color: hexToRgb(icecubeColorHex),
        };

      } else if (selectedAnimId === "rainbow") {
        params = {
          angle: animAngle,
          palette: animPalette,
        };
      } else if (selectedAnimId === "fire") {
        params = {
          intensity: fireIntensity,
        };
      } else if (selectedAnimId === "metaballs") {
        params = {
          count: metaballCount,
          ball_size: metaballSize,
          palette: animPalette,
        };
      } else if (selectedAnimId === "aurora") {
        params = {
          size: auroraSize,
        };
      } else if (selectedAnimId === "pinwheel") {
        params = {
          count: pinwheelArms,
          palette: animPalette,
        };
      } else if (selectedAnimId === "twinkle") {
        params = {
          count: twinkleCount,
          palette: animPalette,
        };
      } else if (selectedAnimId === "breathe") {
        params = {
          palette: animPalette,
          color: hexToRgb(breatheColorHex),
        };
      }

      modePayload = {
        kind: "anim" as const,
        id: selectedAnimId,
        speed: animSpeed,
        params,
      };
    } else if (selectedModeKind === "raw") {
      // ponytail: Doom raw streaming mode
      modePayload = { kind: "raw" as const };
    } else if (selectedModeKind === "loop") {
      modePayload = {
        kind: "loop" as const,
        ids: loopEffectIds,
        interval_secs: loopInterval,
      };
    } else {
      return;
    }
    await setMode(modePayload);
    settings = await getSettings();

    // ponytail: toggle Doom frame stream
    if (selectedModeKind === "raw") {
      startDoom();
    } else {
      isDoomActive = false;
    }
  }

  function toggleLoopEffect(id: string, checked: boolean) {
    if (checked) {
      if (!loopEffectIds.includes(id)) {
        loopEffectIds = [...loopEffectIds, id];
      }
    } else {
      loopEffectIds = loopEffectIds.filter((x) => x !== id);
    }
    applyMode();
  }

  async function handlePickImage() {
    const path = await pickFile("image");
    if (path) {
      selectedImage = path;
      await applyMode();
    }
  }

  async function handlePickGif() {
    const path = await pickFile("gif");
    if (path) {
      selectedGif = path;
      await applyMode();
    }
  }

  async function updateBrightness(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value);
    await setBrightness(val);
  }

  async function updateRotation(e: Event) {
    const val = parseInt((e.target as HTMLSelectElement).value);
    await setRotation(val);
  }

  async function updateFps(e: Event) {
    const val = parseInt((e.target as HTMLSelectElement).value);
    await setFps(val);
  }

  async function toggleAutostart() {
    await setAutostart(autostart);
  }

  async function toggleMinimizeToTray() {
    await setMinimizeToTray(minimizeToTray);
  }

  async function toggleStartMinimized() {
    await setStartMinimized(startMinimized);
  }

  async function handleResolveConflict() {
    resolvingConflict = true;
    try {
      await stopCmService();
      setTimeout(async () => {
        status = await getStatus();
        resolvingConflict = false;
      }, 3000);
    } catch (e) {
      alert("Failed to stop Cooler Master service. Please make sure you are running as Administrator.");
      resolvingConflict = false;
    }
  }
</script>

<main class="app-container">
  <!-- Left Side: Status and Live Hexagon Preview -->
  <section class="preview-panel">
    <div class="header">
      <div class="logo">Atmos Pixel</div>
      <div class="badge-container">
        {#if status}
          {#if status.conflict}
            <span class="badge warning animate-pulse">Device In Use</span>
          {:else if status.device_present}
            <span class="badge success">Connected</span>
          {:else}
            <span class="badge danger">Disconnected</span>
          {/if}
        {/if}
      </div>
    </div>

    <!-- Hexagon Canvas rendering 556 LEDs -->
    <HexPreview />

    {#if status?.conflict}
      <div class="conflict-alert">
        <p>Cooler Master MasterCTRL is owning the exclusive connection to the display.</p>
        <button class="btn btn-warning" onclick={handleResolveConflict} disabled={resolvingConflict}>
          {#if resolvingConflict}
            Resolving...
          {:else}
            Deactivate MasterCTRL Agent
          {/if}
        </button>
      </div>
    {:else if status && !status.device_present}
      <div class="device-alert">
        <p>Hexagonal LED screen is not detected. Please verify your Cooler Master pump's USB connection.</p>
      </div>
    {/if}
  </section>

  <!-- Right Side: Control Panels & Settings -->
  <section class="control-panel">
    <!-- Tabs Navigation -->
    <nav class="tabs">
      <button class="tab-btn" class:active={activeTab === "modes"} onclick={() => (activeTab = "modes")}>
        Display Modes
      </button>
      <button class="tab-btn" class:active={activeTab === "settings"} onclick={() => (activeTab = "settings")}>
        Settings & Info
      </button>
    </nav>

    <!-- Tab Content -->
    <div class="tab-content">
      {#if activeTab === "modes"}
        <!-- Modes Section -->
        <div class="mode-selector">
          <label class="control-label" for="mode-kind">Select Mode</label>
          <div class="select-wrapper">
            <select id="mode-kind" bind:value={selectedModeKind} onchange={applyMode}>
              <option value="off">Off (Blank)</option>
              <option value="solid">Solid Color</option>
              <option value="text">Scrolling Text</option>
              <option value="image">Static Image</option>
              <option value="gif">GIF Animation</option>
              <option value="anim">Procedural Effect</option>
              <option value="loop">Animation Loop</option>
              <option value="raw">DOOM Game</option>
            </select>
          </div>
        </div>

        <div class="mode-options card">
          {#if selectedModeKind === "off"}
            <div class="empty-state">
              <p>Display is turned off. Press another mode to turn it on.</p>
            </div>
          {:else if selectedModeKind === "solid"}
            <SolidControl bind:solidColorHex onchange={applyMode} />
          {:else if selectedModeKind === "text"}
            <TextControl
              bind:textString
              bind:textColorHex
              bind:textScroll
              onapply={applyMode}
            />
          {:else if selectedModeKind === "image"}
            <MediaControl kind="image" bind:selectedPath={selectedImage} onpick={handlePickImage} />
          {:else if selectedModeKind === "gif"}
            <MediaControl kind="gif" bind:selectedPath={selectedGif} onpick={handlePickGif} />
          {:else if selectedModeKind === "anim"}
            <AnimControl
              {animations}
              bind:selectedAnimId
              bind:animSpeed
              bind:starfieldDensity
              bind:starfieldTrailLen
              bind:animPalette
              bind:animAngle
              bind:fireIntensity
              bind:metaballCount
              bind:metaballSize
              bind:auroraSize
              bind:twinkleCount
              bind:pinwheelArms
              bind:breatheColorHex
              bind:rubikAngleX
              bind:rubikAngleY
              bind:rubikScale
              bind:rubikPanX
              bind:rubikPanY
              bind:icecubeAngleX
              bind:icecubeAngleY
              bind:icecubeScale
              bind:icecubePanX
              bind:icecubePanY
              bind:icecubeColorHex
              bind:bounceBallSize
              bind:juliaZoom
              onchange={applyMode}
            />
          {:else if selectedModeKind === "loop"}
            <!-- Loop Mode UI -->
            <div class="loop-control">
              <h3>Animation Loop</h3>
              <p class="loop-desc">Cycle through your selected procedural effects automatically.</p>
              
              <div class="form-group mt-4">
                <div class="slider-header">
                  <label class="control-label" for="loop-interval">Interval (seconds)</label>
                  <span class="slider-val">{loopInterval}s</span>
                </div>
                <input
                  type="range"
                  id="loop-interval"
                  min="2"
                  max="60"
                  step="1"
                  bind:value={loopInterval}
                  onchange={applyMode}
                />
              </div>

              <div class="effects-grid mt-4">
                <span class="control-label">Select Effects to Include:</span>
                <div class="checkbox-list">
                  {#each animations as anim}
                    <label class="checkbox-item">
                      <input
                        type="checkbox"
                        checked={loopEffectIds.includes(anim.id)}
                        onchange={(e) => toggleLoopEffect(anim.id, (e.target as HTMLInputElement).checked)}
                      />
                      <span class="checkbox-label">{anim.label}</span>
                    </label>
                  {/each}
                </div>
              </div>
            </div>
          {:else if selectedModeKind === "raw"}
            <!-- ponytail: render Doom play area -->
            <div class="doom-control">
              <h3>DOOM (Shareware)</h3>
              <p class="doom-desc">Play actual DOOM! Keyboard inputs are captured when this window is active. Frames are scaled and sent to the LED screen in real-time.</p>
              <div class="doom-controls-help">
                <span><strong>Move:</strong> Arrow Keys</span>
                <span><strong>Shoot:</strong> Ctrl</span>
                <span><strong>Use/Open:</strong> Space</span>
              </div>
              <div class="canvas-wrapper">
                <canvas id="doom-canvas" width="640" height="400"></canvas>
              </div>
            </div>
          {/if}
        </div>
      {:else}
        <!-- Settings Section -->
        <SettingsControl
          bind:brightness
          bind:rotation
          bind:fps
          bind:autostart
          bind:minimizeToTray
          bind:startMinimized
          onbrightness={updateBrightness}
          onrotation={updateRotation}
          onfps={updateFps}
          onautostart={toggleAutostart}
          onminimizeToTray={toggleMinimizeToTray}
          onstartMinimized={toggleStartMinimized}
        />

        <div class="app-info mt-6 text-center">
          <p class="version">cm-pixel-gui v1.0.0</p>
          <p class="license">Licensed under MIT License</p>
        </div>
      {/if}
    </div>
  </section>
</main>

<style>
  @import url("https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700&display=swap");

  :global(body) {
    margin: 0;
    font-family: "Outfit", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background-color: #08080a;
    color: #e5e5ed;
    -webkit-font-smoothing: antialiased;
    overflow: hidden;
  }

  .app-container {
    display: grid;
    grid-template-columns: 360px 1fr;
    height: 100vh;
    background-color: #08080a;
  }

  /* Left Panel */
  .preview-panel {
    display: flex;
    flex-direction: column;
    background-color: #0b0b0e;
    border-right: 1px solid rgba(255, 255, 255, 0.04);
    padding: 30px 20px;
    box-sizing: border-box;
    justify-content: flex-start;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 40px;
  }

  .logo {
    font-weight: 700;
    font-size: 1.3rem;
    letter-spacing: -0.5px;
    background: linear-gradient(135deg, #007aff, #00c781);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .badge {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 20px;
    letter-spacing: 0.2px;
  }

  .badge.success {
    background-color: rgba(0, 199, 129, 0.1);
    color: #00c781;
    border: 1px solid rgba(0, 199, 129, 0.2);
  }

  .badge.warning {
    background-color: rgba(255, 179, 0, 0.1);
    color: #ffb300;
    border: 1px solid rgba(255, 179, 0, 0.2);
  }

  .badge.danger {
    background-color: rgba(255, 59, 48, 0.1);
    color: #ff3b30;
    border: 1px solid rgba(255, 59, 48, 0.2);
  }

  .conflict-alert,
  .device-alert {
    background-color: rgba(255, 179, 0, 0.03);
    border: 1px solid rgba(255, 179, 0, 0.1);
    border-radius: 12px;
    padding: 16px;
    margin-top: 30px;
    font-size: 0.85rem;
    text-align: center;
    color: #a1a1b0;
  }

  .conflict-alert p,
  .device-alert p {
    margin: 0 0 14px 0;
    line-height: 1.4;
  }

  .device-alert {
    background-color: rgba(255, 59, 48, 0.03);
    border-color: rgba(255, 59, 48, 0.1);
  }

  /* Right Panel */
  .control-panel {
    display: flex;
    flex-direction: column;
    padding: 30px 40px;
    box-sizing: border-box;
    overflow-y: auto;
  }

  .tabs {
    display: flex;
    gap: 8px;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    padding: 4px;
    border-radius: 12px;
    margin-bottom: 30px;
    width: fit-content;
  }

  .tab-btn {
    background: transparent;
    border: none;
    color: #8c8c9a;
    padding: 8px 18px;
    font-size: 0.9rem;
    font-weight: 500;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .tab-btn.active {
    background-color: rgba(255, 255, 255, 0.06);
    color: #ffffff;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .tab-content {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
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

  .mode-selector {
    margin-bottom: 24px;
  }

  /* Form Elements */
  .card {
    background-color: #0b0b0e;
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 16px;
    padding: 24px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
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

  /* Buttons */
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

  .btn-warning {
    background-color: rgba(255, 179, 0, 0.15);
    color: #ffb300;
    border: 1px solid rgba(255, 179, 0, 0.3);
  }

  .btn-warning:hover {
    background-color: rgba(255, 179, 0, 0.25);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty-state {
    text-align: center;
    color: rgba(255, 255, 255, 0.3);
    padding: 20px 0;
    font-style: italic;
    font-size: 0.9rem;
  }

  /* Info */
  .app-info {
    font-size: 0.8rem;
    color: #575762;
  }

  .app-info p {
    margin: 4px 0;
  }

  .mt-6 {
    margin-top: 24px;
  }
  .animate-pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  /* ponytail: DOOM layout styles */
  .doom-control {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .doom-control h3 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .doom-desc {
    font-size: 0.9rem;
    color: #8c8c9a;
    line-height: 1.4;
    margin: 0;
  }

  .doom-controls-help {
    display: flex;
    gap: 16px;
    font-size: 0.85rem;
    color: #e5e5ed;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    padding: 10px 14px;
    border-radius: 8px;
  }

  .canvas-wrapper {
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #000000;
    border-radius: 12px;
    overflow: hidden;
    aspect-ratio: 8/5;
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  #doom-canvas {
    width: 100%;
    height: 100%;
    display: block;
  }

  /* Loop Mode UI styles */
  .loop-control {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .loop-control h3 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .loop-desc {
    font-size: 0.9rem;
    color: #8c8c9a;
    line-height: 1.4;
    margin: 0;
  }

  .effects-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .checkbox-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 10px;
    max-height: 250px;
    overflow-y: auto;
    background-color: rgba(255, 255, 255, 0.01);
    border: 1px solid rgba(255, 255, 255, 0.05);
    padding: 12px;
    border-radius: 10px;
  }

  .checkbox-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    color: #e5e5ed;
    cursor: pointer;
    user-select: none;
    padding: 6px 8px;
    border-radius: 6px;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.03);
    transition: background-color 0.2s, border-color 0.2s;
  }

  .checkbox-item:hover {
    background-color: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.08);
  }

  .checkbox-item input[type="checkbox"] {
    accent-color: #007aff;
    cursor: pointer;
    width: 14px;
    height: 14px;
  }

  .checkbox-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
