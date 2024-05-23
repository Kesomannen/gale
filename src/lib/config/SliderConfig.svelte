<script lang="ts">
	import { setTaggedConfig } from "$lib/invoke";
	import type { ConfigValue, ConfigNum, ConfigEntryId, ConfigRange } from "$lib/models";
	import { Slider } from "bits-ui";
	import ResetConfigButton from "./ResetConfigButton.svelte";

    export let entryId: ConfigEntryId;

    let content = entryId.entry.value.content as ConfigNum;
    let type = entryId.entry.value.type as 'int32' | 'double' | 'single';
    let range = content.range as ConfigRange;

    
    function onReset(newValue: ConfigValue) {
        content = newValue.content as ConfigNum;
    }

    function onValueChange(newValue: number) {
        content.value = newValue;
        setTaggedConfig(entryId, { type, content });
    }
</script>


<input
	type="text"
	bind:value={content.value}
	class="py-1 w-14 ml-2 rounded-md bg-gray-900 text-sm text-center
		text-slate-400 hover:text-slate-100 border border-gray-500 border-opacity-0 hover:border-opacity-100"
/>
<ResetConfigButton {entryId} {onReset} />