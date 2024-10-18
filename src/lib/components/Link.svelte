<script lang="ts">
	export let href: string;
</script>

<a
	{href}
	target="_blank"
	rel="noopener noreferrer"
	class="text-green-400 hover:text-green-300 hover:underline"
>
	<slot />
</a>
