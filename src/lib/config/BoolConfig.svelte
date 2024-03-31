<script lang="ts">
	import { invokeCommand } from "$lib/invoke";
	import type { ConfigEntry, ConfigValue } from "$lib/models";
	import { Checkbox, Label, Select, Switch } from "bits-ui";
	import ResetConfigButton from "./ResetConfigButton.svelte";
	import Icon from "@iconify/svelte";

    export let file: string;
    export let section: string;
    export let entry: ConfigEntry;

    let value: boolean = entry.value.content as boolean;

    function onReset(newValue: ConfigValue) {
        value = newValue.content as boolean;
    }

    $: {
        let configValue: ConfigValue = {
            type: "boolean",
            content: value
        }

        invokeCommand('set_config_entry', { file, section, entry: entry.name, value: configValue });
    }
</script>

<div class="flex items-center flex-grow">
    <Checkbox.Root 
        id="checkbox"
        bind:checked={value}
    >
        <Checkbox.Indicator
            class="rounded-md w-5 h-5 p-1 
            bg-{value ? 'green-700' : 'gray-800'}
            hover:bg-{value ? 'green-600' : 'gray-700'}
            {value ? '' : 'border border-gray-500'}"
        >
            {#if value}
                <Icon class="text-white w-full h-full font-bold" icon="mdi:check" />
            {/if}
        </Checkbox.Indicator>
    </Checkbox.Root>
</div>
<ResetConfigButton {file} {section} {entry} {onReset} />