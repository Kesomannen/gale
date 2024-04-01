<script lang="ts">
	import type { ConfigEntryId, ConfigValue, SelectItem } from "$lib/models";
	import { Select } from "bits-ui";
	import ResetConfigButton from "./ResetConfigButton.svelte";
	import { slide } from "svelte/transition";
	import Icon from "@iconify/svelte";
	import { setConfig } from "$lib/invoke";

    export let entryId: ConfigEntryId;

    let content = entryId.entry.value.content as { value: string, options: string[] };
    let items = content.options.map(valueToItem);

    let selectedItem = valueToItem(content.value);

    function valueToItem(value: string): SelectItem {
        return { value: value, label: value };
    }

    function onReset(newValue: ConfigValue) {
        content = newValue.content as { value: string, options: string[] };
        selectedItem = valueToItem(content.value);
    }

    function onSelectChange(value: string) {
        setConfig(entryId, {
            type: 'enum',
            content: {
                value,
                options: content.options
            }
        });
    }
</script>

<Select.Root 
    {items}
    bind:selected={selectedItem}
    onSelectedChange={selected => {
        if (selected) {
            onSelectChange(selected.value);
        }
    }}
>
    <Select.Trigger
        class="flex items-center flex-grow bg-gray-900 rounded-lg px-3 py-1 text-sm
                border border-gray-500 border-opacity-0 hover:border-opacity-100"
    >
        <Select.Value class="text-slate-300 text-left w-full" />
        <Icon class="text-slate-400 text-xl ml-auto" icon="mdi:chevron-down" />
    </Select.Trigger>
    <Select.Content
        class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-600"
        transition={slide}
        transitionConfig={{ duration: 100 }}
    >
        {#each items as item}
            <Select.Item
                value={item.value}
                class="flex items-center px-3 py-1 truncate text-sm text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
            >
                {item.label}
            </Select.Item>
        {/each}
    </Select.Content>
</Select.Root>
<ResetConfigButton {entryId} {onReset} />