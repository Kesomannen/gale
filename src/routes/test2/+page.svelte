<script lang="ts">
  import { profiles } from '$lib/state/profile.svelte';
</script>

<div class="text-white">
  {#each profiles.active?.mods ?? [] as mod}
    <div>{mod.name}</div>
  {/each}
</div>
