<script lang="ts">
	import Markdown from '$lib/components/ui/Markdown.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import type { MarkdownType, Mod } from '$lib/types';
	import Icon from '@iconify/svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import { getMarkdown } from '$lib/util';

	type Props = {
		open?: boolean;
		useLatest?: boolean;
		mod: Mod;
		type: MarkdownType;
	};

	let { open = $bindable(false), useLatest = false, mod, type }: Props = $props();

	let promise: Promise<string | null> | null = $state(null);
	let currentMod: Mod | null = null;

	export async function fetchMarkdown() {
		if (currentMod === mod) return;
		currentMod = mod;

		promise = getMarkdown(mod, type, useLatest);
	}
</script>

<Dialog large bind:open>
	{#await promise}
		<Icon class="text-primary-300 animate-spin text-4xl" icon="mdi:loading" />
	{:then value}
		{#if value !== null}
			<Markdown source={value} />
		{:else}
			<div class="text-primary-300 flex items-center justify-center gap-2">
				{m.modInfoDialog_noFound({ type })}
			</div>
		{/if}
	{:catch error}
		<div class="flex items-center justify-center gap-2 text-red-400">
			<Icon class="text-lg" icon="mdi:alert-circle-outline" />
			{m.modInfoDIalog_failed({ type, error })}
		</div>
	{/await}
</Dialog>
