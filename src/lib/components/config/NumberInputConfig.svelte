<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigNum, ConfigValue } from '$lib/types';
	import ResetConfigButton from './ResetConfigButton.svelte';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	// svelte-ignore state_referenced_locally (local editing state seeded from prop)
	let content = $state(entryId.entry.value.content as ConfigNum);
	let type = $derived(entryId.entry.value.type as 'int' | 'float');

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
	onchange={submit}
	class="focus:ring-accent-600! text-primary-700 placeholder-primary-500 enabled:hover:ring-primary-400 disabled:text-primary-500 dark:focus:ring-accent-500! dark:bg-primary-900 dark:text-primary-300 dark:placeholder-primary-400 dark:enabled:hover:ring-primary-500 dark:disabled:text-primary-400 bg-primary-100 w-full grow rounded-lg px-3 py-1 focus:ring-2! focus:outline-hidden enabled:hover:ring-1"
/>

<ResetConfigButton {entryId} {onReset} {locked} />

<style>
	input::-webkit-inner-spin-button,
	input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
