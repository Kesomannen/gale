<script lang="ts">
	import Markdown from '$lib/components/ui/Markdown.svelte';
	import Popup from '$lib/components/ui/Popup.svelte';
	import type { Mod } from '$lib/types';
	import Icon from '@iconify/svelte';
	import * as api from '$lib/api';

	type Props = {
		open?: boolean;
		useLatest?: boolean;
		mod: Mod;
		kind: 'readme' | 'changelog';
	};

	let { open = $bindable(false), useLatest = false, mod, kind }: Props = $props();

	let promise: Promise<string | null> | null = $state(null);
	let currentMod: Mod | null = null;

	export async function fetchMarkdown() {
		if (currentMod === mod) return;
		currentMod = mod;

		let modRef = {
			packageUuid: mod.uuid,
			versionUuid: useLatest ? mod.versions[0].uuid : mod.versionUuid
		};

		promise = api.thunderstore.getMarkdown(modRef, kind);
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
