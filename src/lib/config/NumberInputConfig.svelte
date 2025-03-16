<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigNum, ConfigValue } from '$lib/models';
	import ResetConfigButton from './ResetConfigButton.svelte';

	export let entryId: ConfigEntryId;

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
	step={type === 'int' ? 1 : 'any'}
	bind:value={content.value}
	on:change={submit}
	class="focus:ring-accent-400 bg-primary-900 text-primary-300 placeholder-primary-400 hover:border-primary-500 w-full grow rounded-lg border border-transparent px-3 py-1 focus:border-transparent focus:ring-2 focus:outline-hidden"
/>

<ResetConfigButton {entryId} {onReset} />

<style>
	input::-webkit-inner-spin-button,
	input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
