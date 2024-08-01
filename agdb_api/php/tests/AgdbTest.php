<?php declare(strict_types=1);
use PHPUnit\Framework\TestCase;
use agdb\QueryBuilder;

final class AgdbTest extends TestCase
{
    public function testAgdb(): void
    {
        $qb = new QueryBuilder();
        $this->assertSame($qb->query, "");
    }
}
