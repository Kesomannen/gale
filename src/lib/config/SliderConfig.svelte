<script lang="ts">
	import { setTaggedConfig } from "$lib/config";
	import type { ConfigValue, ConfigNum, ConfigEntryId, ConfigRange } from "$lib/models";
	import { Slider } from "bits-ui";
	import ResetConfigButton from "./ResetConfigButton.svelte";

    export let entryId: ConfigEntryId;

    let content = entryId.entry.value.content as ConfigNum;
    let type = entryId.entry.value.type as 'int32' | 'double' | 'single';
    let range = content.range as ConfigRange;
    $: sliderValue = [content.value];
    
    function onReset(newValue: ConfigValue) {
        content = newValue.content as ConfigNum;
    }

    function onValueChange(newValue: number) {
        content.value = newValue;
        setTaggedConfig(entryId, { type, content });
    }
</script>

<Slider.Root 
    let:thumbs 
    bind:value={sliderValue}
    onValueChange={values => onValueChange(values[0])}
    min={range.start}
    max={range.end}
    step={type === 'int32' ? 1 : 0.01}
    class="flex flex-grow relative items-center group transition-none"
>
    <div class="bg-gray-900 h-2 rounded-full w-full">
        <Slider.Range class="h-full rounded-full bg-gray-700" />
    </div>
    {#each thumbs as thumb}
        <Slider.Thumb {thumb} class="h-4 w-4 rounded-full bg-gray-500 group-hover:bg-gray-300" />
    {/each}
</Slider.Root>

<input
	type="text"
	bind:value={content.value}
	class="py-1 w-14 ml-2 rounded-md bg-gray-900 text-sm text-center
		text-slate-400 hover:text-slate-100 border border-gray-500 border-opacity-0 hover:border-opacity-100"
/>
<ResetConfigButton {entryId} {onReset} />