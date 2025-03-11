<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigNum, ConfigValue } from '$lib/models';
	import ResetConfigButton from './ResetConfigButton.svelte';

	export let entryId: ConfigEntryId;
	export let locked: boolean;

	let value = entryId.entry.value;
	let content = value.content as ConfigNum;
	let type = value.type as 'int' | 'float';

	function onReset(value: ConfigValue) {
		content = value.content as ConfigNum;
	}

	function submit() {
		setConfigEntry(entryId, { type, content });
	}
</script>

<input
	type="number"
	disabled={locked}
	step={type === 'int' ? 1 : 'any'}
	bind:value={content.value}
	on:change={submit}
	class="focus:ring-accent-400 w-full grow rounded-lg border border-transparent bg-slate-900 px-3 py-1 text-slate-300 placeholder-slate-400 hover:border-slate-500 focus:border-transparent focus:ring-2 focus:outline-hidden"
/>

<ResetConfigButton {entryId} {onReset} {locked} />

<style>
	input::-webkit-inner-spin-button,
	input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
