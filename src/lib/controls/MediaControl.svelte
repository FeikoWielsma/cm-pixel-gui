<script lang="ts">
  let {
    kind,
    selectedPath = $bindable(),
    onpick,
  } = $props<{
    kind: "image" | "gif";
    selectedPath: string;
    onpick: () => void;
  }>();
</script>

<div class="media-selector">
  <button class="btn btn-outline" onclick={onpick}>
    Select {kind === "image" ? "Image" : "GIF"} File
  </button>
  {#if selectedPath}
    <div class="file-path truncate" title={selectedPath}>
      {selectedPath.split(/[\\/]/).pop()}
    </div>
  {:else}
    <div class="file-path empty">No file selected</div>
  {/if}
</div>

<style>
  .media-selector {
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
    text-align: center;
  }
  .file-path {
    font-size: 0.85rem;
    color: #8c8c9a;
    max-width: 100%;
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
</style>
