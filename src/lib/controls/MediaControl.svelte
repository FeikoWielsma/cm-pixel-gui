<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { HistoryItem } from "../api";

  let {
    kind,
    selectedPath = $bindable(),
    onpick,
    history = [],
    ontogglefavorite,
    ondeletehistory,
    onselecthistory,
  } = $props<{
    kind: "image" | "gif";
    selectedPath: string;
    onpick: () => void;
    history?: HistoryItem[];
    ontogglefavorite?: (path: string) => void;
    ondeletehistory?: (path: string) => void;
    onselecthistory?: (path: string) => void;
  }>();

  function getFilename(path: string): string {
    return path.split(/[\\/]/).pop() || "";
  }
</script>

<div class="media-selector">
  <button class="btn btn-outline" onclick={onpick}>
    Select {kind === "image" ? "Image" : "GIF"} File
  </button>
  {#if selectedPath}
    <div class="file-path truncate" title={selectedPath}>
      Active: {getFilename(selectedPath)}
    </div>
  {:else}
    <div class="file-path empty">No file selected</div>
  {/if}

  {#if history && history.length > 0}
    <div class="history-section mt-4">
      <span class="history-title">Recent Gallery</span>
      <div class="history-list">
        <div class="hex-grid">
          {#each history as item}
            <div class="hex-wrapper">
              <div 
                role="button"
                tabindex="0"
                class="hex-cell" 
                class:active={selectedPath === item.path}
                class:has-fav={item.favorite}
                onclick={() => onselecthistory?.(item.path)}
                onkeydown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    onselecthistory?.(item.path);
                  }
                }}
                title={getFilename(item.path)}
              >
                <img 
                  src={convertFileSrc(item.path)} 
                  alt="thumbnail" 
                  class="hex-thumb" 
                />
                <div class="hex-overlay">
                  <div class="hex-action-row" style="justify-content: flex-end;">
                    <button 
                      class="action-btn fav-btn" 
                      class:is-fav={item.favorite}
                      onclick={(e) => {
                        e.stopPropagation();
                        ontogglefavorite?.(item.path);
                      }}
                      title={item.favorite ? "Unfavorite" : "Favorite"}
                    >
                      ★
                    </button>
                  </div>
                  <div class="hex-action-row" style="justify-content: center;">
                    <button 
                      class="action-btn del-btn" 
                      onclick={(e) => {
                        e.stopPropagation();
                        ondeletehistory?.(item.path);
                      }}
                      title="Remove"
                    >
                      ✕
                    </button>
                  </div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .media-selector {
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
    text-align: center;
    width: 100%;
  }
  .file-path {
    font-size: 0.85rem;
    color: #8c8c9a;
    max-width: 100%;
    margin-top: 4px;
  }
  .file-path.empty {
    font-style: italic;
    color: rgba(255, 255, 255, 0.2);
  }
  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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
  .btn-outline {
    background-color: transparent;
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #e5e5ed;
  }
  .btn-outline:hover {
    background-color: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.2);
  }

  /* History list styles */
  .history-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    margin-top: 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    padding-top: 16px;
    text-align: left;
  }
  .history-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.4);
    margin-bottom: 4px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .history-list {
    max-height: 250px;
    overflow-y: auto;
    width: 100%;
  }

  /* Hex grid layout styles */
  .hex-grid {
    display: grid;
    grid-template-columns: repeat(4, 60px);
    grid-auto-rows: 70px;
    column-gap: 0;
    row-gap: 0;
    justify-content: center;
    padding-top: 15px;
    padding-bottom: 50px;
    margin: 0 auto;
  }

  .hex-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    position: relative;
    width: 80px;
    height: 70px;
  }

  /* Shift even columns (Col 1 and Col 3) */
  .hex-wrapper:nth-child(4n + 2),
  .hex-wrapper:nth-child(4n + 4) {
    transform: translateY(35px);
  }

  .hex-cell {
    position: relative;
    width: 80px;
    height: 70px;
    background-color: rgba(255, 255, 255, 0.03);
    clip-path: polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%);
    cursor: pointer;
    transition: transform 0.2s, filter 0.2s;
    overflow: hidden;
    box-sizing: border-box;
    border: none;
    padding: 0;
    margin: 0;
  }

  .hex-cell:hover {
    transform: scale(1.05);
    z-index: 10;
  }

  .hex-cell.active {
    filter: drop-shadow(0 0 5px #007aff) brightness(1.1);
    transform: scale(1.05);
    z-index: 10;
  }

  .hex-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    background-color: #08080a;
  }

  .hex-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.6);
    opacity: 0;
    transition: opacity 0.2s;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 12px 6px;
    box-sizing: border-box;
  }

  .hex-cell:hover .hex-overlay,
  .hex-cell.has-fav .hex-overlay {
    opacity: 1;
  }

  .hex-cell.has-fav:not(:hover) .hex-overlay {
    background: transparent;
    pointer-events: none;
  }

  .hex-cell.has-fav:not(:hover) .del-btn {
    display: none;
  }

  .hex-cell.has-fav:not(:hover) .fav-btn {
    color: #ffb703;
    text-shadow: 0 0 4px rgba(0, 0, 0, 0.9);
  }

  .hex-action-row {
    display: flex;
    width: 100%;
    pointer-events: auto;
  }

  .action-btn {
    background: none;
    border: none;
    padding: 2px;
    margin: 0;
    font-size: 0.9rem;
    cursor: pointer;
    line-height: 1;
    color: rgba(255, 255, 255, 0.4);
    transition: color 0.2s, transform 0.2s;
    font-family: inherit;
  }

  .action-btn:hover {
    transform: scale(1.2);
  }

  .fav-btn:hover {
    color: #ffb703;
  }
  .fav-btn.is-fav {
    color: #ffb703;
  }
  .del-btn:hover {
    color: #ff4d4f;
  }
</style>
