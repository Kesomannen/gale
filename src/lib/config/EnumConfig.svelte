<script lang="ts">
	import InputField from "$lib/InputField.svelte";
	import { invokeCommand } from "$lib/invoke";
	import type { ConfigEntry, ConfigValue } from "$lib/models";
	import { Select } from "bits-ui";
	import ResetConfigButton from "./ResetConfigButton.svelte";
	import { slide } from "svelte/transition";
	import Icon from "@iconify/svelte";

    export let file: string;
    export let section: string;
    export let entry: ConfigEntry;

    let content = entry.value.content as { value: string, options: string[] };
    $: options = content.options;

    let selectedItem = { value: content.value, label: content.value };
    $: items = options.map((option) => ({ value: option, label: option }));

    function onReset(newValue: ConfigValue) {
        let entryValue = newValue.content as { value: string, options: string[] }
        let value = entryValue.value;
        selectedItem = { value, label: value };
    }

    $: {
        let configValue: ConfigValue = {
            type: 'enum',
            content: {
                value: selectedItem.value,
                options
            }
        }

        invokeCommand('set_config_entry', { file, section, entry: entry.name, value: configValue });
    }
</script>

<Select.Root {items} bind:selected={selectedItem}>
    <Select.Trigger
        class="flex items-center flex-grow bg-gray-900 rounded-lg px-3 py-1 text-sm
                border border-gray-500 border-opacity-0 hover:border-opacity-100"
    >
        <Select.Value class="text-slate-300 text-left w-full" />
        <Icon class="text-slate-400 text-xl ml-auto" icon="mdi:chevron-down" />
    </Select.Trigger>
    <Select.Content
        class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-2 rounded-lg border border-gray-600"
        transition={slide}
        transitionConfig={{ duration: 100 }}
    >
        {#each options as option}
            <Select.Item
                value={option}
                class="flex items-center px-3 py-1 text-slate-400 hover:text-slate-200 text-left rounded-lg hover:bg-gray-700 cursor-default"
            >
                {option}
            </Select.Item>
        {/each}
    </Select.Content>
</Select.Root>
<ResetConfigButton {file} {section} {entry} {onReset} />