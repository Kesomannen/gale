<script lang="ts">
	import Checkbox from "$lib/components/Checkbox.svelte";
	import Label from "$lib/components/Label.svelte";
	import { invokeCommand } from "$lib/invoke";
	import type { PrefValue } from "$lib/models";
	import { dialog } from "@tauri-apps/api";
	import { onMount } from "svelte";

    export let label: string;
    export let key: string;
    export let disableMessage: string | undefined = undefined;

    let value: boolean;

    onMount(async () => {
		value = (await invokeCommand<PrefValue>('get_pref', { key })) as boolean;
	});

	async function onValueChanged(newValue: boolean) {
        if (!newValue && disableMessage) {
            
            let confirmed = await dialog.confirm(disableMessage);
            if (!confirmed) {
                value = true;
                return;
            }
        }
        
		value = newValue;
		invokeCommand('set_pref', { key, value });
	}
</script>

<div class="flex items-center my-0.5">
	<Label text={label}>
		<slot />
	</Label>

	<Checkbox bind:value {onValueChanged} />
</div>
