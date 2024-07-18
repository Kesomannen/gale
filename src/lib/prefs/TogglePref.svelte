<script lang="ts">
	import Checkbox from "$lib/components/Checkbox.svelte";
	import Label from "$lib/components/Label.svelte";
	import { invokeCommand } from "$lib/invoke";
	import { confirm } from "@tauri-apps/plugin-dialog";
	import { onMount } from "svelte";

    export let label: string;
    export let disableMessage: string | null = null;
	
	export let value: boolean;
	export let set: (value: boolean) => void;

	async function onValueChanged(newValue: boolean) {
        if (!newValue && disableMessage) {
            let confirmed = await confirm(disableMessage);
            if (!confirmed) {
                value = true;
                return;
            }
        }
        
		value = newValue;
		set(newValue);
	}
</script>

<div class="flex items-center my-0.5">
	<Label text={label}>
		<slot />
	</Label>

	<Checkbox bind:value {onValueChanged} />
</div>
