<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigNum, ConfigValue } from '$lib/models';
	import ResetConfigButton from './ResetConfigButton.svelte';

	export let entryId: ConfigEntryId;

	let value = entryId.entry.value;
	let content = value.content as ConfigNum;
	let type = value.type as 'int32' | 'double' | 'single';

	function onReset(value: ConfigValue) {
		content = value.content as ConfigNum;
	}

	$: setConfigEntry(entryId, { type, content });
</script>

<input
	type="number"
	step={type === 'int32' ? 1 : 'any'}
	bind:value={content.value}
	class="w-full flex-grow rounded-lg border border-slate-500 border-opacity-0 bg-slate-900 px-3 py-1 text-slate-300 placeholder-slate-400 hover:border-opacity-100 focus:border-opacity-0 focus:outline-none focus:ring-2 focus:ring-accent-400"
/>

<ResetConfigButton {entryId} {onReset} />

<style>
	input::-webkit-inner-spin-button,
	input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
