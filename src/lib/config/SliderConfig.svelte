<script lang="ts">
	import InputField from "$lib/InputField.svelte";
	import { invokeCommand } from "$lib/invoke";
	import type { ConfigEntry, ConfigValue, ConfigNum as ConfigNum } from "$lib/models";
	import { Slider } from "bits-ui";
	import ResetConfigButton from "./ResetConfigButton.svelte";

    export let file: string;
    export let section: string;
    export let entry: ConfigEntry;

    let value = entry.value.content as ConfigNum;
    $: type = entry.value.type;

    function onReset(newValue: ConfigValue) {
        value = newValue.content as ConfigNum;
    }

    $: {
        let configValue: ConfigValue = {
            type,
            content: value
        }

        invokeCommand('set_config_entry', { file, section, entry: entry.name, value: configValue });
    }
</script>

<Slider.Root>

</Slider.Root>
<ResetConfigButton {file} {section} {entry} {onReset} />