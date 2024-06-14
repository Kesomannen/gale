<script lang="ts">
	import Markdown from "$lib/components/Markdown.svelte";
	import Popup from "$lib/components/Popup.svelte";
	import type { MarkdownResponse, Mod } from "$lib/models";
	import Icon from "@iconify/svelte";
	import { clipboard } from "@tauri-apps/api";
 	import { Response, fetch } from "@tauri-apps/api/http";

	export let open = false;
	export let useLatest = false;
	export let mod: Mod;
	export let path: string;

	let promise: Promise<Response<MarkdownResponse>> | null = null;
	let currentMod: Mod | null = null;
	
	export function fetchMarkdown() {
		if (currentMod === mod) return;
		currentMod = mod;

		let version = useLatest ? mod.versions[0].name : mod.version;

		let url = `https://thunderstore.io/api/experimental/package/${mod.author}/${mod.name}/${version}/${path}/`;
		promise = fetch<MarkdownResponse>(url)
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