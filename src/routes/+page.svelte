<script lang="ts">
	import Markdown from "$lib/Markdown.svelte";
	import { onMount } from "svelte";

  let changelogPromise: Promise<string>;

  onMount(async () => {
    let response = await fetch("https://raw.githubusercontent.com/Kesomannen/ModManager/master/CHANGELOG.md");
    changelogPromise = response.text();
  });
</script>

<div class="px-6 overflow-y-auto text-slate-100">
  {#await changelogPromise}
    Loading changelog...
  {:then changelog}
    <Markdown source={changelog} />
  {:catch error}
    Failed to load changelog: {error.message}
  {/await}
</div>