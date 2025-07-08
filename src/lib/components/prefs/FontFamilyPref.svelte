<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import Combobox from '$lib/components/ui/Combobox.svelte';
	import ResetButton from '$lib/components/ui/ResetButton.svelte';
	import { selectItems } from '$lib/util';
	import { getFont, setFont } from '$lib/theme';
	import { onMount } from 'svelte';
	import * as api from '$lib/api';

	let fonts: string[] = $state([]);
	let value = $state(getFont());

	onMount(async () => {
		fonts = ['Nunito Sans', ...(await api.prefs.getSystemFonts())];
	});

	$effect(() => {
		setFont(value);
	});
</script>

<div class="flex items-center">
	<Label>Font family</Label>

	<Combobox
		items={selectItems(fonts)}
		type="single"
		triggerClass="grow"
		placeholder="Search for a font..."
		bind:value
	/>

	<ResetButton onclick={() => (value = 'Nunito Sans')} class="ml-1" />
</div>
