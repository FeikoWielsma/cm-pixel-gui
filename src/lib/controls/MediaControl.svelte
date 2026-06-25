<script lang="ts">
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
      <span class="history-title">Recent Files</span>
      <div class="history-list">
        {#each history as item}
          <div class="history-item" class:active={selectedPath === item.path}>
            <button 
              class="history-path-btn truncate" 
              onclick={() => onselecthistory?.(item.path)}
              title={item.path}
            >
              {getFilename(item.path)}
            </button>
            <div class="history-actions">
              <button 
                class="action-btn fav-btn" 
                class:is-fav={item.favorite}
                onclick={() => ontogglefavorite?.(item.path)}
                title={item.favorite ? "Unfavorite" : "Favorite"}
              >
                ★
              </button>
              <button 
                class="action-btn del-btn" 
                onclick={() => ondeletehistory?.(item.path)}
                title="Remove from history"
              >
                ✕
              </button>
            </div>
          </div>
        {/each}
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
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 180px;
    overflow-y: auto;
    padding-right: 4px;
  }
  .history-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.03);
    border-radius: 8px;
    padding: 6px 10px;
    transition: all 0.2s ease;
    width: 100%;
    box-sizing: border-box;
  }
  .history-item:hover {
    background-color: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.08);
  }
  .history-item.active {
    background-color: rgba(0, 122, 255, 0.1);
    border-color: rgba(0, 122, 255, 0.3);
  }
  .history-path-btn {
    background: none;
    border: none;
    padding: 0;
    margin: 0;
    color: #c5c5d2;
    font-size: 0.85rem;
    cursor: pointer;
    text-align: left;
    flex-grow: 1;
    max-width: calc(100% - 60px);
    font-family: inherit;
  }
  .history-item.active .history-path-btn {
    color: #ffffff;
    font-weight: 600;
  }
  .history-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .action-btn {
    background: none;
    border: none;
    padding: 4px;
    margin: 0;
    font-size: 0.9rem;
    cursor: pointer;
    line-height: 1;
    color: rgba(255, 255, 255, 0.3);
    transition: color 0.2s;
    font-family: inherit;
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
