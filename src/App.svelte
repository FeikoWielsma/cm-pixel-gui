<script lang="ts">
  import { onMount } from "svelte";
  import { getStatus } from "./lib/api";
  import type { Status } from "./lib/api";

  let status: Status | null = $state(null);

  onMount(async () => {
    status = await getStatus();
  });
</script>

<main>
  <h1>CM Pixel GUI</h1>
  {#if status}
    <p>Device: {status.device_present ? "connected" : "not found"}</p>
  {:else}
    <p>Loading…</p>
  {/if}
</main>
