<script lang="ts">
	import Markdown from '$lib/components/Markdown.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { Mod } from '$lib/models';
	import Icon from '@iconify/svelte';

	export let open = false;
	export let useLatest = false;
	export let mod: Mod;
	export let kind: 'readme' | 'changelog';

	let promise: Promise<string | null> | null = null;
	let currentMod: Mod | null = null;

	export async function fetchMarkdown() {
		if (currentMod === mod) return;
		currentMod = mod;

		let modRef = {
			packageUuid: mod.uuid,
			versionUuid: useLatest ? mod.versions[0].uuid : mod.versionUuid
		};

		promise = invokeCommand('get_markdown', { modRef, kind });
	}
</script>

<Popup large bind:open>
	{#await promise}
		<Icon class="text-primary-300 animate-spin text-4xl" icon="mdi:loading" />
	{:then value}
		{#if value !== null}
			<Markdown source={value} />
		{:else}
			<div class="text-primary-300 flex items-center justify-center gap-2">
				No {kind} found
			</div>
		{/if}
	{:catch error}
		<div class="flex items-center justify-center gap-2 text-red-400">
			<Icon class="text-lg" icon="mdi:alert-circle-outline" />
			Failed to load {kind}: {error}
		</div>
	{/await}
</Popup>
