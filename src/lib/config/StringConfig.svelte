<script lang="ts">
	import InputField from "$lib/InputField.svelte";
	import { invokeCommand } from "$lib/invoke";
	import type { ConfigEntry, ConfigValue } from "$lib/models";
	import ResetConfigButton from "./ResetConfigButton.svelte";

    export let file: string;
    export let section: string;
    export let entry: ConfigEntry;

    let value: string = entry.value.content as string;

    function onReset(newValue: ConfigValue) {
        value = newValue.content as string;
    }

    $: {
        let configValue: ConfigValue = {
            type: "string",
            content: value
        }

        invokeCommand('set_config_entry', { file, section, entry: entry.name, value: configValue });
    }
</script>

<InputField bind:value={value} />
<ResetConfigButton {file} {section} {entry} {onReset} />