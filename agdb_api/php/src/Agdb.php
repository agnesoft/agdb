<?php
use Agnesoft\AgdbApi\Model\QueryResult;

class Agdb
{
    public static function try_into(string $type, QueryResult $result): array
    {
        $t = new $type();
        $reflect = new ReflectionClass($t);
        $props = [];
        $ar = [];

        foreach ($reflect->getProperties() as $prop) {
            $props[$prop->getName()] = $prop->getType()->getName();
        }

        foreach ($result->getElements() as $element) {
            $e = new $type();
            $e->db_id = $element->getId();

            foreach ($element->getValues() as $kv) {
                $key_name = $kv->getKey()->getString();
                $value_type = $props[$key_name];

                if ($value_type === "string") {
                    $e->$key_name = $kv->getValue()->getString();
                } elseif ($value_type === "int") {
                    $e->$key_name = $kv->getValue()->getI64();
                } elseif ($value_type === "float") {
                    $e->$key_name = $kv->getValue()->getF64();
                } elseif ($value_type === "bool") {
                    $v = $kv->getValue()->getString();
                    $e->$key_name = $v === "true" ? true : false;
                } elseif ($value_type === "array") {
                    $v = $v = $kv->getValue()->getVecString();
                    if ($v === null) {
                        $v = $kv->getValue()->getVecI64();
                        if ($v === null) {
                            $v = $kv->getValue()->getVecF64();
                        }
                    }
                    $e->$key_name = $v;
                }
            }

            $ar[] = $e;
        }

        return $ar;
    }
}
