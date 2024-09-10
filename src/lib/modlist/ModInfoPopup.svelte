<script lang="ts">
	import Markdown from '$lib/components/Markdown.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import type { MarkdownResponse, Mod } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { fetch } from '@tauri-apps/plugin-http';

	export let open = false;
	export let useLatest = false;
	export let mod: Mod;
	export let path: string;

	let promise: Promise<MarkdownResponse> | null = null;
	let currentMod: Mod | null = null;

	export async function fetchMarkdown() {
		if (currentMod === mod) return;
		currentMod = mod;

		let version = useLatest ? mod.versions[0].name : mod.version;

		let url = `https://thunderstore.io/api/experimental/package/${mod.author}/${mod.name}/${version}/${path}/`;
		promise = fetch(url).then((res) => res.json()) as Promise<MarkdownResponse>;
	}
</script>

<Popup bind:open>
	{#await promise}
		<Icon class="text-slate-300 text-4xl animate-spin" icon="mdi:loading" />
	{:then value}
		{#if value !== null}
			{#if value.markdown !== undefined}
				<Markdown source={value.markdown} />
			{:else}
				<p class="text-red-300">No {path} found</p>
			{/if}
		{:else}
			<p class="text-red-300">Failed to load {path}</p>
		{/if}
	{:catch error}
		<p class="text-red-300">Failed to load {path}: {error}</p>
	{/await}
</Popup>
