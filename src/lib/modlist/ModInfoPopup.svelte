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

<Popup large bind:open>
	{#await promise}
		<Icon class="animate-spin text-4xl text-gray-300" icon="mdi:loading" />
	{:then value}
		{#if value !== null}
			{#if value.markdown}
				<Markdown source={value.markdown} />
			{:else}
				<div class="flex items-center justify-center gap-2 text-gray-300">
					No {path} found 😥
				</div>
			{/if}
		{:else}
			<div class="flex items-center justify-center gap-2 text-red-400">
				<Icon class="text-lg" icon="mdi:alert-circle-outline" />
				Failed to load {path}
			</div>
		{/if}
	{:catch error}
		<div class="flex items-center justify-center gap-2 text-red-400">
			<Icon class="text-lg" icon="mdi:alert-circle-outline" />
			Failed to load {path}: {error}
		</div>
	{/await}
</Popup>
