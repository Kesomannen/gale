<script lang="ts">
	import Markdown from "$lib/components/Markdown.svelte";
	import Popup from "$lib/components/Popup.svelte";
	import type { Mod } from "$lib/models";
	import Icon from "@iconify/svelte";
  import { Response, fetch } from "@tauri-apps/api/http";

  export let open = false;
  export let mod: Mod;
  export let path: string;

	let promise: Promise<Response<MarkdownResponse>> | null = null;
	let currentMod: Mod | null = null;

  type MarkdownResponse = { 
    markdown?: string;
    detail?: string;
  }

	export function fetchMarkdown() {
		if (currentMod === mod) return;
		currentMod = mod;

		let url = `https://thunderstore.io/api/experimental/package/${mod.author}/${mod.name}/${mod.version}/${path}/`;
		promise = fetch<MarkdownResponse>(url, { method: 'GET' })
	}
</script>

<Popup bind:open>
	{#await promise}
		<Icon class="text-slate-300 text-4xl animate-spin" icon="mdi:loading" />
	{:then value}
		{#if value?.ok}
      {#if value?.data.markdown}
        <Markdown source={value.data.markdown} />
      {:else}
        <p class="text-red-300">No {path} found</p>
      {/if}
		{:else}
			<p class="text-red-300">Failed to load {path}: error code {value?.status}</p>
		{/if}
	{:catch error}
		<p class="text-red-300">Failed to load {path}: {error}</p>
	{/await}
</Popup>