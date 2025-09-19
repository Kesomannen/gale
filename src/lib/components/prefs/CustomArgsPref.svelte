<script lang="ts">
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '@iconify/svelte';

	type Props = {
		value: string[] | null;
		set: (value: string[] | null) => Promise<void>;
	};

	let { value = $bindable(), set }: Props = $props();

	let newArg = $state('');
</script>

<div class="mt-1 flex items-center">
	<Label>{m.customArgsPref_title()}</Label>

	<Info>
		<p>
			{m.customArgsPref_content_1()}<b>{m.customArgsPref_content_2()}</b>{m.customArgsPref_content_3()}
		</p>

		<p>
			{m.customArgsPref_content_4()}
			<code>--foo value</code>
			{m.customArgsPref_content_5()}
			<code>--foo</code>
			{m.customArgsPref_content_6()}
			<code>value</code>
			{m.customArgsPref_content_7()}
		</p>
	</Info>

	<Checkbox
		checked={value !== null}
		onCheckedChange={(newValue) => {
			set(newValue ? [] : null);
		}}
	/>
</div>

{#if value !== null}
	<div class="text-primary-300 mt-1 flex flex-col gap-1 pl-[35%]">
		{#each value as argument, i}
			<div class="flex gap-1">
				<button
					class="text-primary-400 hover:bg-primary-700 hover:text-primary-300 rounded-lg p-1.5 text-xl"
					onclick={() => {
						if (value === null) return;
						value = value.filter((_, index) => index !== i);
						set(value);
					}}
				>
					<Icon icon="mdi:remove" />
				</button>
				<InputField
					class="grow"
					value={argument}
					onchange={(newValue) => {
						if (value === null) return;
						value[i] = newValue;
						set(value);
					}}
				/>
			</div>
		{/each}

		<InputField
			placeholder={m.customArgsPref_placeholder()}
			bind:value={newArg}
			onchange={() => {
				if (newArg.length === 0 || value === null) return;

				value = [...value, newArg];
				newArg = '';
				set(value);
			}}
		/>
	</div>
{/if}
