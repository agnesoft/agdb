<?php

namespace Agnesoft\Agdb\Normalizer;

use Agnesoft\Agdb\Runtime\Normalizer\CheckArray;
use Agnesoft\Agdb\Runtime\Normalizer\ValidatorTrait;
use Symfony\Component\Serializer\Normalizer\DenormalizerAwareInterface;
use Symfony\Component\Serializer\Normalizer\DenormalizerAwareTrait;
use Symfony\Component\Serializer\Normalizer\DenormalizerInterface;
use Symfony\Component\Serializer\Normalizer\NormalizerAwareInterface;
use Symfony\Component\Serializer\Normalizer\NormalizerAwareTrait;
use Symfony\Component\Serializer\Normalizer\NormalizerInterface;
use Symfony\Component\HttpKernel\Kernel;
if (!class_exists(Kernel::class) or (Kernel::MAJOR_VERSION >= 7 or Kernel::MAJOR_VERSION === 6 and Kernel::MINOR_VERSION === 4)) {
    class JaneObjectNormalizer implements DenormalizerInterface, NormalizerInterface, DenormalizerAwareInterface, NormalizerAwareInterface
    {
        use DenormalizerAwareTrait;
        use NormalizerAwareTrait;
        use CheckArray;
        use ValidatorTrait;
        protected $normalizers = [
            
            \Agnesoft\Agdb\Model\ChangePassword::class => \Agnesoft\Agdb\Normalizer\ChangePasswordNormalizer::class,
            
            \Agnesoft\Agdb\Model\ClusterStatus::class => \Agnesoft\Agdb\Normalizer\ClusterStatusNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbElement::class => \Agnesoft\Agdb\Normalizer\DbElementNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbKeyValue::class => \Agnesoft\Agdb\Normalizer\DbKeyValueNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbTypeParam::class => \Agnesoft\Agdb\Normalizer\DbTypeParamNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbUser::class => \Agnesoft\Agdb\Normalizer\DbUserNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbUserRoleParam::class => \Agnesoft\Agdb\Normalizer\DbUserRoleParamNormalizer::class,
            
            \Agnesoft\Agdb\Model\InsertAliasesQuery::class => \Agnesoft\Agdb\Normalizer\InsertAliasesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\InsertEdgesQuery::class => \Agnesoft\Agdb\Normalizer\InsertEdgesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\InsertNodesQuery::class => \Agnesoft\Agdb\Normalizer\InsertNodesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\InsertValuesQuery::class => \Agnesoft\Agdb\Normalizer\InsertValuesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\QueryAudit::class => \Agnesoft\Agdb\Normalizer\QueryAuditNormalizer::class,
            
            \Agnesoft\Agdb\Model\QueryCondition::class => \Agnesoft\Agdb\Normalizer\QueryConditionNormalizer::class,
            
            \Agnesoft\Agdb\Model\QueryResult::class => \Agnesoft\Agdb\Normalizer\QueryResultNormalizer::class,
            
            \Agnesoft\Agdb\Model\SearchQuery::class => \Agnesoft\Agdb\Normalizer\SearchQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\SelectEdgeCountQuery::class => \Agnesoft\Agdb\Normalizer\SelectEdgeCountQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\SelectValuesQuery::class => \Agnesoft\Agdb\Normalizer\SelectValuesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\ServerDatabase::class => \Agnesoft\Agdb\Normalizer\ServerDatabaseNormalizer::class,
            
            \Agnesoft\Agdb\Model\ServerDatabaseRename::class => \Agnesoft\Agdb\Normalizer\ServerDatabaseRenameNormalizer::class,
            
            \Agnesoft\Agdb\Model\ServerDatabaseResource::class => \Agnesoft\Agdb\Normalizer\ServerDatabaseResourceNormalizer::class,
            
            \Agnesoft\Agdb\Model\StatusParams::class => \Agnesoft\Agdb\Normalizer\StatusParamsNormalizer::class,
            
            \Agnesoft\Agdb\Model\UserCredentials::class => \Agnesoft\Agdb\Normalizer\UserCredentialsNormalizer::class,
            
            \Agnesoft\Agdb\Model\UserLogin::class => \Agnesoft\Agdb\Normalizer\UserLoginNormalizer::class,
            
            \Agnesoft\Agdb\Model\UserStatus::class => \Agnesoft\Agdb\Normalizer\UserStatusNormalizer::class,
            
            \Jane\Component\JsonSchemaRuntime\Reference::class => \Agnesoft\Agdb\Runtime\Normalizer\ReferenceNormalizer::class,
        ], $normalizersCache = [];
        public function supportsDenormalization($data, $type, $format = null, array $context = []): bool
        {
            return array_key_exists($type, $this->normalizers);
        }
        public function supportsNormalization($data, $format = null, array $context = []): bool
        {
            return is_object($data) && array_key_exists(get_class($data), $this->normalizers);
        }
        public function normalize(mixed $object, string $format = null, array $context = []): array|string|int|float|bool|\ArrayObject|null
        {
            $normalizerClass = $this->normalizers[get_class($object)];
            $normalizer = $this->getNormalizer($normalizerClass);
            return $normalizer->normalize($object, $format, $context);
        }
        public function denormalize(mixed $data, string $type, string $format = null, array $context = []): mixed
        {
            $denormalizerClass = $this->normalizers[$type];
            $denormalizer = $this->getNormalizer($denormalizerClass);
            return $denormalizer->denormalize($data, $type, $format, $context);
        }
        private function getNormalizer(string $normalizerClass)
        {
            return $this->normalizersCache[$normalizerClass] ?? $this->initNormalizer($normalizerClass);
        }
        private function initNormalizer(string $normalizerClass)
        {
            $normalizer = new $normalizerClass();
            $normalizer->setNormalizer($this->normalizer);
            $normalizer->setDenormalizer($this->denormalizer);
            $this->normalizersCache[$normalizerClass] = $normalizer;
            return $normalizer;
        }
        public function getSupportedTypes(?string $format = null): array
        {
            return [
                
                \Agnesoft\Agdb\Model\ChangePassword::class => false,
                \Agnesoft\Agdb\Model\ClusterStatus::class => false,
                \Agnesoft\Agdb\Model\DbElement::class => false,
                \Agnesoft\Agdb\Model\DbKeyValue::class => false,
                \Agnesoft\Agdb\Model\DbTypeParam::class => false,
                \Agnesoft\Agdb\Model\DbUser::class => false,
                \Agnesoft\Agdb\Model\DbUserRoleParam::class => false,
                \Agnesoft\Agdb\Model\InsertAliasesQuery::class => false,
                \Agnesoft\Agdb\Model\InsertEdgesQuery::class => false,
                \Agnesoft\Agdb\Model\InsertNodesQuery::class => false,
                \Agnesoft\Agdb\Model\InsertValuesQuery::class => false,
                \Agnesoft\Agdb\Model\QueryAudit::class => false,
                \Agnesoft\Agdb\Model\QueryCondition::class => false,
                \Agnesoft\Agdb\Model\QueryResult::class => false,
                \Agnesoft\Agdb\Model\SearchQuery::class => false,
                \Agnesoft\Agdb\Model\SelectEdgeCountQuery::class => false,
                \Agnesoft\Agdb\Model\SelectValuesQuery::class => false,
                \Agnesoft\Agdb\Model\ServerDatabase::class => false,
                \Agnesoft\Agdb\Model\ServerDatabaseRename::class => false,
                \Agnesoft\Agdb\Model\ServerDatabaseResource::class => false,
                \Agnesoft\Agdb\Model\StatusParams::class => false,
                \Agnesoft\Agdb\Model\UserCredentials::class => false,
                \Agnesoft\Agdb\Model\UserLogin::class => false,
                \Agnesoft\Agdb\Model\UserStatus::class => false,
                \Jane\Component\JsonSchemaRuntime\Reference::class => false,
            ];
        }
    }
} else {
    class JaneObjectNormalizer implements DenormalizerInterface, NormalizerInterface, DenormalizerAwareInterface, NormalizerAwareInterface
    {
        use DenormalizerAwareTrait;
        use NormalizerAwareTrait;
        use CheckArray;
        use ValidatorTrait;
        protected $normalizers = [
            
            \Agnesoft\Agdb\Model\ChangePassword::class => \Agnesoft\Agdb\Normalizer\ChangePasswordNormalizer::class,
            
            \Agnesoft\Agdb\Model\ClusterStatus::class => \Agnesoft\Agdb\Normalizer\ClusterStatusNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbElement::class => \Agnesoft\Agdb\Normalizer\DbElementNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbKeyValue::class => \Agnesoft\Agdb\Normalizer\DbKeyValueNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbTypeParam::class => \Agnesoft\Agdb\Normalizer\DbTypeParamNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbUser::class => \Agnesoft\Agdb\Normalizer\DbUserNormalizer::class,
            
            \Agnesoft\Agdb\Model\DbUserRoleParam::class => \Agnesoft\Agdb\Normalizer\DbUserRoleParamNormalizer::class,
            
            \Agnesoft\Agdb\Model\InsertAliasesQuery::class => \Agnesoft\Agdb\Normalizer\InsertAliasesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\InsertEdgesQuery::class => \Agnesoft\Agdb\Normalizer\InsertEdgesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\InsertNodesQuery::class => \Agnesoft\Agdb\Normalizer\InsertNodesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\InsertValuesQuery::class => \Agnesoft\Agdb\Normalizer\InsertValuesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\QueryAudit::class => \Agnesoft\Agdb\Normalizer\QueryAuditNormalizer::class,
            
            \Agnesoft\Agdb\Model\QueryCondition::class => \Agnesoft\Agdb\Normalizer\QueryConditionNormalizer::class,
            
            \Agnesoft\Agdb\Model\QueryResult::class => \Agnesoft\Agdb\Normalizer\QueryResultNormalizer::class,
            
            \Agnesoft\Agdb\Model\SearchQuery::class => \Agnesoft\Agdb\Normalizer\SearchQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\SelectEdgeCountQuery::class => \Agnesoft\Agdb\Normalizer\SelectEdgeCountQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\SelectValuesQuery::class => \Agnesoft\Agdb\Normalizer\SelectValuesQueryNormalizer::class,
            
            \Agnesoft\Agdb\Model\ServerDatabase::class => \Agnesoft\Agdb\Normalizer\ServerDatabaseNormalizer::class,
            
            \Agnesoft\Agdb\Model\ServerDatabaseRename::class => \Agnesoft\Agdb\Normalizer\ServerDatabaseRenameNormalizer::class,
            
            \Agnesoft\Agdb\Model\ServerDatabaseResource::class => \Agnesoft\Agdb\Normalizer\ServerDatabaseResourceNormalizer::class,
            
            \Agnesoft\Agdb\Model\StatusParams::class => \Agnesoft\Agdb\Normalizer\StatusParamsNormalizer::class,
            
            \Agnesoft\Agdb\Model\UserCredentials::class => \Agnesoft\Agdb\Normalizer\UserCredentialsNormalizer::class,
            
            \Agnesoft\Agdb\Model\UserLogin::class => \Agnesoft\Agdb\Normalizer\UserLoginNormalizer::class,
            
            \Agnesoft\Agdb\Model\UserStatus::class => \Agnesoft\Agdb\Normalizer\UserStatusNormalizer::class,
            
            \Jane\Component\JsonSchemaRuntime\Reference::class => \Agnesoft\Agdb\Runtime\Normalizer\ReferenceNormalizer::class,
        ], $normalizersCache = [];
        public function supportsDenormalization($data, $type, $format = null, array $context = []): bool
        {
            return array_key_exists($type, $this->normalizers);
        }
        public function supportsNormalization($data, $format = null, array $context = []): bool
        {
            return is_object($data) && array_key_exists(get_class($data), $this->normalizers);
        }
        /**
         * @return array|string|int|float|bool|\ArrayObject|null
         */
        public function normalize($object, $format = null, array $context = [])
        {
            $normalizerClass = $this->normalizers[get_class($object)];
            $normalizer = $this->getNormalizer($normalizerClass);
            return $normalizer->normalize($object, $format, $context);
        }
        /**
         * @return mixed
         */
        public function denormalize($data, $type, $format = null, array $context = [])
        {
            $denormalizerClass = $this->normalizers[$type];
            $denormalizer = $this->getNormalizer($denormalizerClass);
            return $denormalizer->denormalize($data, $type, $format, $context);
        }
        private function getNormalizer(string $normalizerClass)
        {
            return $this->normalizersCache[$normalizerClass] ?? $this->initNormalizer($normalizerClass);
        }
        private function initNormalizer(string $normalizerClass)
        {
            $normalizer = new $normalizerClass();
            $normalizer->setNormalizer($this->normalizer);
            $normalizer->setDenormalizer($this->denormalizer);
            $this->normalizersCache[$normalizerClass] = $normalizer;
            return $normalizer;
        }
        public function getSupportedTypes(?string $format = null): array
        {
            return [
                
                \Agnesoft\Agdb\Model\ChangePassword::class => false,
                \Agnesoft\Agdb\Model\ClusterStatus::class => false,
                \Agnesoft\Agdb\Model\DbElement::class => false,
                \Agnesoft\Agdb\Model\DbKeyValue::class => false,
                \Agnesoft\Agdb\Model\DbTypeParam::class => false,
                \Agnesoft\Agdb\Model\DbUser::class => false,
                \Agnesoft\Agdb\Model\DbUserRoleParam::class => false,
                \Agnesoft\Agdb\Model\InsertAliasesQuery::class => false,
                \Agnesoft\Agdb\Model\InsertEdgesQuery::class => false,
                \Agnesoft\Agdb\Model\InsertNodesQuery::class => false,
                \Agnesoft\Agdb\Model\InsertValuesQuery::class => false,
                \Agnesoft\Agdb\Model\QueryAudit::class => false,
                \Agnesoft\Agdb\Model\QueryCondition::class => false,
                \Agnesoft\Agdb\Model\QueryResult::class => false,
                \Agnesoft\Agdb\Model\SearchQuery::class => false,
                \Agnesoft\Agdb\Model\SelectEdgeCountQuery::class => false,
                \Agnesoft\Agdb\Model\SelectValuesQuery::class => false,
                \Agnesoft\Agdb\Model\ServerDatabase::class => false,
                \Agnesoft\Agdb\Model\ServerDatabaseRename::class => false,
                \Agnesoft\Agdb\Model\ServerDatabaseResource::class => false,
                \Agnesoft\Agdb\Model\StatusParams::class => false,
                \Agnesoft\Agdb\Model\UserCredentials::class => false,
                \Agnesoft\Agdb\Model\UserLogin::class => false,
                \Agnesoft\Agdb\Model\UserStatus::class => false,
                \Jane\Component\JsonSchemaRuntime\Reference::class => false,
            ];
        }
    }
}