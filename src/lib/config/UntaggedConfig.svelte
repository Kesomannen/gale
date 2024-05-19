<script lang="ts">
	import InputField from '$lib/InputField.svelte';
	import { setUntaggedConfig } from '$lib/invoke';
	import type { ConfigFile, ConfigSection } from '$lib/models';

	export let file: ConfigFile;
	export let section: ConfigSection;
	export let name: string;
  export let value: string;

	$: setUntaggedConfig(file, section, name, value);
</script>

<InputField bind:value size='sm' />
