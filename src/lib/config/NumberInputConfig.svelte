<script lang="ts">
	import { setTaggedConfig } from "$lib/invoke";
	import type { ConfigEntryId, ConfigNum, ConfigValue } from "$lib/models";
	import ResetConfigButton from "./ResetConfigButton.svelte";

  export let entryId: ConfigEntryId;

  let value = entryId.entry.value;
  let content = value.content as ConfigNum;
  let type = value.type as 'int32' | 'double' | 'single';

  function onReset(value: ConfigValue) {
    content = value.content as ConfigNum;
  }

  $: setTaggedConfig(entryId, { type, content });
</script>

<input
	type="number"
  step={type === 'int32' ? 1 : 'any'}
  bind:value={content.value}
	class="flex-grow px-3 py-1 rounded-lg bg-gray-900 text-sm
	 text-slate-300 hover:text-slate-100 border border-gray-500 border-opacity-0 hover:border-opacity-100"
/>

<ResetConfigButton {entryId} {onReset} />

<style>
  input::-webkit-inner-spin-button,
  input::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
</style>