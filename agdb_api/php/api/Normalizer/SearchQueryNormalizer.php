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
    class SearchQueryNormalizer implements DenormalizerInterface, NormalizerInterface, DenormalizerAwareInterface, NormalizerAwareInterface
    {
        use DenormalizerAwareTrait;
        use NormalizerAwareTrait;
        use CheckArray;
        use ValidatorTrait;
        public function supportsDenormalization(mixed $data, string $type, string $format = null, array $context = []): bool
        {
            return $type === \Agnesoft\Agdb\Model\SearchQuery::class;
        }
        public function supportsNormalization(mixed $data, string $format = null, array $context = []): bool
        {
            return is_object($data) && get_class($data) === \Agnesoft\Agdb\Model\SearchQuery::class;
        }
        public function denormalize(mixed $data, string $type, string $format = null, array $context = []): mixed
        {
            if (isset($data['$ref'])) {
                return new Reference($data['$ref'], $context['document-origin']);
            }
            if (isset($data['$recursiveRef'])) {
                return new Reference($data['$recursiveRef'], $context['document-origin']);
            }
            $object = new \Agnesoft\Agdb\Model\SearchQuery();
            if (null === $data || false === \is_array($data)) {
                return $object;
            }
            if (\array_key_exists('algorithm', $data)) {
                $object->setAlgorithm($data['algorithm']);
                unset($data['algorithm']);
            }
            if (\array_key_exists('conditions', $data)) {
                $values = [];
                foreach ($data['conditions'] as $value) {
                    $values[] = $this->denormalizer->denormalize($value, \Agnesoft\Agdb\Model\QueryCondition::class, 'json', $context);
                }
                $object->setConditions($values);
                unset($data['conditions']);
            }
            if (\array_key_exists('destination', $data)) {
                $object->setDestination($data['destination']);
                unset($data['destination']);
            }
            if (\array_key_exists('limit', $data)) {
                $object->setLimit($data['limit']);
                unset($data['limit']);
            }
            if (\array_key_exists('offset', $data)) {
                $object->setOffset($data['offset']);
                unset($data['offset']);
            }
            if (\array_key_exists('order_by', $data)) {
                $values_1 = [];
                foreach ($data['order_by'] as $value_1) {
                    $values_1[] = $value_1;
                }
                $object->setOrderBy($values_1);
                unset($data['order_by']);
            }
            if (\array_key_exists('origin', $data)) {
                $object->setOrigin($data['origin']);
                unset($data['origin']);
            }
            foreach ($data as $key => $value_2) {
                if (preg_match('/.*/', (string) $key)) {
                    $object[$key] = $value_2;
                }
            }
            return $object;
        }
        public function normalize(mixed $object, string $format = null, array $context = []): array|string|int|float|bool|\ArrayObject|null
        {
            $data = [];
            $data['algorithm'] = $object->getAlgorithm();
            $values = [];
            foreach ($object->getConditions() as $value) {
                $values[] = $this->normalizer->normalize($value, 'json', $context);
            }
            $data['conditions'] = $values;
            $data['destination'] = $object->getDestination();
            $data['limit'] = $object->getLimit();
            $data['offset'] = $object->getOffset();
            $values_1 = [];
            foreach ($object->getOrderBy() as $value_1) {
                $values_1[] = $value_1;
            }
            $data['order_by'] = $values_1;
            $data['origin'] = $object->getOrigin();
            foreach ($object as $key => $value_2) {
                if (preg_match('/.*/', (string) $key)) {
                    $data[$key] = $value_2;
                }
            }
            return $data;
        }
        public function getSupportedTypes(?string $format = null): array
        {
            return [\Agnesoft\Agdb\Model\SearchQuery::class => false];
        }
    }
} else {
    class SearchQueryNormalizer implements DenormalizerInterface, NormalizerInterface, DenormalizerAwareInterface, NormalizerAwareInterface
    {
        use DenormalizerAwareTrait;
        use NormalizerAwareTrait;
        use CheckArray;
        use ValidatorTrait;
        public function supportsDenormalization($data, $type, string $format = null, array $context = []): bool
        {
            return $type === \Agnesoft\Agdb\Model\SearchQuery::class;
        }
        public function supportsNormalization(mixed $data, string $format = null, array $context = []): bool
        {
            return is_object($data) && get_class($data) === \Agnesoft\Agdb\Model\SearchQuery::class;
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
            $object = new \Agnesoft\Agdb\Model\SearchQuery();
            if (null === $data || false === \is_array($data)) {
                return $object;
            }
            if (\array_key_exists('algorithm', $data)) {
                $object->setAlgorithm($data['algorithm']);
                unset($data['algorithm']);
            }
            if (\array_key_exists('conditions', $data)) {
                $values = [];
                foreach ($data['conditions'] as $value) {
                    $values[] = $this->denormalizer->denormalize($value, \Agnesoft\Agdb\Model\QueryCondition::class, 'json', $context);
                }
                $object->setConditions($values);
                unset($data['conditions']);
            }
            if (\array_key_exists('destination', $data)) {
                $object->setDestination($data['destination']);
                unset($data['destination']);
            }
            if (\array_key_exists('limit', $data)) {
                $object->setLimit($data['limit']);
                unset($data['limit']);
            }
            if (\array_key_exists('offset', $data)) {
                $object->setOffset($data['offset']);
                unset($data['offset']);
            }
            if (\array_key_exists('order_by', $data)) {
                $values_1 = [];
                foreach ($data['order_by'] as $value_1) {
                    $values_1[] = $value_1;
                }
                $object->setOrderBy($values_1);
                unset($data['order_by']);
            }
            if (\array_key_exists('origin', $data)) {
                $object->setOrigin($data['origin']);
                unset($data['origin']);
            }
            foreach ($data as $key => $value_2) {
                if (preg_match('/.*/', (string) $key)) {
                    $object[$key] = $value_2;
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
            $data['algorithm'] = $object->getAlgorithm();
            $values = [];
            foreach ($object->getConditions() as $value) {
                $values[] = $this->normalizer->normalize($value, 'json', $context);
            }
            $data['conditions'] = $values;
            $data['destination'] = $object->getDestination();
            $data['limit'] = $object->getLimit();
            $data['offset'] = $object->getOffset();
            $values_1 = [];
            foreach ($object->getOrderBy() as $value_1) {
                $values_1[] = $value_1;
            }
            $data['order_by'] = $values_1;
            $data['origin'] = $object->getOrigin();
            foreach ($object as $key => $value_2) {
                if (preg_match('/.*/', (string) $key)) {
                    $data[$key] = $value_2;
                }
            }
            return $data;
        }
        public function getSupportedTypes(?string $format = null): array
        {
            return [\Agnesoft\Agdb\Model\SearchQuery::class => false];
        }
    }
}