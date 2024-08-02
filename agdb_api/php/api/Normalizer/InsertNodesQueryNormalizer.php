<?php

namespace Agnesoft\Agdb\Normalizer;

use Jane\Component\JsonSchemaRuntime\Reference;
use Agnesoft\Agdb\Runtime\Normalizer\CheckArray;
use Agnesoft\Agdb\Runtime\Normalizer\ValidatorTrait;
use Symfony\Component\Serializer\Exception\InvalidArgumentException;
use Symfony\Component\Serializer\Normalizer\DenormalizerAwareInterface;
use Symfony\Component\Serializer\Normalizer\DenormalizerAwareTrait;
use Symfony\Component\Serializer\Normalizer\DenormalizerInterface;
use Symfony\Component\Serializer\Normalizer\NormalizerAwareInterface;
use Symfony\Component\Serializer\Normalizer\NormalizerAwareTrait;
use Symfony\Component\Serializer\Normalizer\NormalizerInterface;
use Symfony\Component\HttpKernel\Kernel;
if (!class_exists(Kernel::class) or (Kernel::MAJOR_VERSION >= 7 or Kernel::MAJOR_VERSION === 6 and Kernel::MINOR_VERSION === 4)) {
    class InsertNodesQueryNormalizer implements DenormalizerInterface, NormalizerInterface, DenormalizerAwareInterface, NormalizerAwareInterface
    {
        use DenormalizerAwareTrait;
        use NormalizerAwareTrait;
        use CheckArray;
        use ValidatorTrait;
        public function supportsDenormalization(mixed $data, string $type, string $format = null, array $context = []): bool
        {
            return $type === \Agnesoft\Agdb\Model\InsertNodesQuery::class;
        }
        public function supportsNormalization(mixed $data, string $format = null, array $context = []): bool
        {
            return is_object($data) && get_class($data) === \Agnesoft\Agdb\Model\InsertNodesQuery::class;
        }
        public function denormalize(mixed $data, string $type, string $format = null, array $context = []): mixed
        {
            if (isset($data['$ref'])) {
                return new Reference($data['$ref'], $context['document-origin']);
            }
            if (isset($data['$recursiveRef'])) {
                return new Reference($data['$recursiveRef'], $context['document-origin']);
            }
            $object = new \Agnesoft\Agdb\Model\InsertNodesQuery();
            if (null === $data || false === \is_array($data)) {
                return $object;
            }
            if (\array_key_exists('aliases', $data)) {
                $values = [];
                foreach ($data['aliases'] as $value) {
                    $values[] = $value;
                }
                $object->setAliases($values);
                unset($data['aliases']);
            }
            if (\array_key_exists('count', $data)) {
                $object->setCount($data['count']);
                unset($data['count']);
            }
            if (\array_key_exists('ids', $data)) {
                $object->setIds($data['ids']);
                unset($data['ids']);
            }
            if (\array_key_exists('values', $data)) {
                $object->setValues($data['values']);
                unset($data['values']);
            }
            foreach ($data as $key => $value_1) {
                if (preg_match('/.*/', (string) $key)) {
                    $object[$key] = $value_1;
                }
            }
            return $object;
        }
        public function normalize(mixed $object, string $format = null, array $context = []): array|string|int|float|bool|\ArrayObject|null
        {
            $data = [];
            $values = [];
            foreach ($object->getAliases() as $value) {
                $values[] = $value;
            }
            $data['aliases'] = $values;
            $data['count'] = $object->getCount();
            $data['ids'] = $object->getIds();
            $data['values'] = $object->getValues();
            foreach ($object as $key => $value_1) {
                if (preg_match('/.*/', (string) $key)) {
                    $data[$key] = $value_1;
                }
            }
            return $data;
        }
        public function getSupportedTypes(?string $format = null): array
        {
            return [\Agnesoft\Agdb\Model\InsertNodesQuery::class => false];
        }
    }
} else {
    class InsertNodesQueryNormalizer implements DenormalizerInterface, NormalizerInterface, DenormalizerAwareInterface, NormalizerAwareInterface
    {
        use DenormalizerAwareTrait;
        use NormalizerAwareTrait;
        use CheckArray;
        use ValidatorTrait;
        public function supportsDenormalization($data, $type, string $format = null, array $context = []): bool
        {
            return $type === \Agnesoft\Agdb\Model\InsertNodesQuery::class;
        }
        public function supportsNormalization(mixed $data, string $format = null, array $context = []): bool
        {
            return is_object($data) && get_class($data) === \Agnesoft\Agdb\Model\InsertNodesQuery::class;
        }
        /**
         * @return mixed
         */
        public function denormalize($data, $type, $format = null, array $context = [])
        {
            if (isset($data['$ref'])) {
                return new Reference($data['$ref'], $context['document-origin']);
            }
            if (isset($data['$recursiveRef'])) {
                return new Reference($data['$recursiveRef'], $context['document-origin']);
            }
            $object = new \Agnesoft\Agdb\Model\InsertNodesQuery();
            if (null === $data || false === \is_array($data)) {
                return $object;
            }
            if (\array_key_exists('aliases', $data)) {
                $values = [];
                foreach ($data['aliases'] as $value) {
                    $values[] = $value;
                }
                $object->setAliases($values);
                unset($data['aliases']);
            }
            if (\array_key_exists('count', $data)) {
                $object->setCount($data['count']);
                unset($data['count']);
            }
            if (\array_key_exists('ids', $data)) {
                $object->setIds($data['ids']);
                unset($data['ids']);
            }
            if (\array_key_exists('values', $data)) {
                $object->setValues($data['values']);
                unset($data['values']);
            }
            foreach ($data as $key => $value_1) {
                if (preg_match('/.*/', (string) $key)) {
                    $object[$key] = $value_1;
                }
            }
            return $object;
        }
        /**
         * @return array|string|int|float|bool|\ArrayObject|null
         */
        public function normalize($object, $format = null, array $context = [])
        {
            $data = [];
            $values = [];
            foreach ($object->getAliases() as $value) {
                $values[] = $value;
            }
            $data['aliases'] = $values;
            $data['count'] = $object->getCount();
            $data['ids'] = $object->getIds();
            $data['values'] = $object->getValues();
            foreach ($object as $key => $value_1) {
                if (preg_match('/.*/', (string) $key)) {
                    $data[$key] = $value_1;
                }
            }
            return $data;
        }
        public function getSupportedTypes(?string $format = null): array
        {
            return [\Agnesoft\Agdb\Model\InsertNodesQuery::class => false];
        }
    }
}