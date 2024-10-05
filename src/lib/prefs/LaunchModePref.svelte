<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import InputField from '$lib/components/InputField.svelte';

	import type { LaunchMode } from '$lib/models';
	import { sentenceCase } from '$lib/util';

	import { t } from '$i18n';

	export let value: LaunchMode;
	export let set: (value: LaunchMode) => Promise<void>;

	let instances = value.content?.instances ?? 1;
	let intervalSecs = value.content?.intervalSecs ?? 10;

	async function onSelectedChangeSingle(newValue: string) {
		value.type = newValue as 'steam' | 'direct';
		await submit();
	}

	async function submit() {
		if (value.type === 'direct') {
			value.content = { instances, intervalSecs };
		} else {
			value.content = undefined;
		}

		await set(value);
	}
</script>

<div class="flex items-center">
	<Label text="{t('Launch mode')}">
		<p>{t('Launch mode description')}</p>
		<p class="my-1.5">
			{@html t('Launch mode description steam')}
		</p>
		<p>
			{@html t('Launch mode description direct')}
		</p>
	</Label>

	<Dropdown
		class="flex-grow"
		items={['steam', 'direct']}
		getLabel={sentenceCase}
		selected={value?.type ?? 'steam'}
		{onSelectedChangeSingle}
	/>
</div>

<div class="flex items-center">
	<Label text="{t("Number of instances")}">
		{t("Number of instances description")}
	</Label>

	<InputField
		disabled={value.type !== 'direct'}
		value={instances.toString()}
		on:change={({ detail }) => {
			instances = parseInt(detail);
			submit();
		}}
	/>
</div>

<div class="flex items-center">
	<Label text="{t("Interval between launches")}">
		{t("Interval between launches description")}
	</Label>

	<InputField
		disabled={value.type !== 'direct' || instances <= 1}
		value={intervalSecs.toString()}
		on:change={({ detail }) => {
			intervalSecs = parseInt(detail);
			submit();
		}}
	/>
</div>
