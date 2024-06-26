<script lang="ts">
    export let href: string;
</script>

<a {href} target="_blank" class="hover:underline text-green-400 hover:text-green-300">
    <slot />
</a>