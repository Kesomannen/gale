<script lang="ts">
	import InputField from '$lib/components/InputField.svelte';
	import { setTaggedConfig } from '$lib/config';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';

	import ResetConfigButton from './ResetConfigButton.svelte';

	export let entryId: ConfigEntryId;
  	export let isOther: boolean = false;

	let content = entryId.entry.value.content as string;

	function onReset(value: ConfigValue) {
		content = value.content as string;
	}

	$: setTaggedConfig(entryId, {
		type: isOther ? 'other' : 'string',
		content: content
	});
</script>

<div class="flex-grow">
	<InputField bind:value={content} size='md' />
</div>
<ResetConfigButton {entryId} {onReset} />
