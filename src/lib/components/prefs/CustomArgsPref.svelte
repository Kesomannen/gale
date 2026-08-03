<script lang="ts">
	import Info from '$lib/components/ui/Info.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		value: string;
		setValue: (value: string) => Promise<void>;
	};

	let { value = $bindable(), setValue }: Props = $props();
</script>

<div class="flex items-center">
	<Label>{m.customArgsPref_title()}</Label>

	<Info>
		<p>
			{m.customArgsPref_content_1()}<b>{m.customArgsPref_content_2()}</b
			>{m.customArgsPref_content_3()}
		</p>
	</Info>

	<InputField
		placeholder={m.customArgsPref_placeholder()}
		bind:value
		onchange={() => setValue(value)}
	/>
</div>
