import { describe, beforeEach, vi, it, expect } from "vitest";
import { useDbDetails, type DbDetailsParams } from "./dbDetails";
import { db_user_list, db_user_add, db_user_remove } from "@/tests/apiMock";
import { ref } from "vue";
import { useDbUsersStore } from "./dbUsersStore";
import useModal from "@/composables/modal/modal";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "../modal/constants";

const dbParams = ref<DbDetailsParams>({
    db: "testDb",
    owner: "testOwner",
    role: "admin",
});
const { fetchDbUsers } = useDbUsersStore();
const { modalIsVisible, onConfirm, closeModal } = useModal();
const { setInputValue, clearAllInputs, getInputValue } = useContentInputs();

describe("dbDetails", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        closeModal();
        clearAllInputs();
    });
    it("should get user list", async () => {
        db_user_list.mockResolvedValue({
            data: [
                {
                    username: "testUser",
                    role: "read",
                },
                {
                    username: "testOwner",
                    role: "admin",
                },
            ],
        });
        const { users } = useDbDetails(dbParams);
        await fetchDbUsers(dbParams.value);
        expect(users.value).toHaveLength(2);
        expect(users.value?.[0].username).toBe("testUser");
    });
    it("should get db name", () => {
        const { dbName } = useDbDetails(dbParams);
        expect(dbName.value).toBe("testOwner/testDb");
    });
    it("should get can edit users", () => {
        const { canEditUsers } = useDbDetails(dbParams);
        expect(canEditUsers.value).toBe(true);
    });
    it("should remove user", () => {
        const { handleRemoveUser } = useDbDetails(dbParams);
        handleRemoveUser("testUser");
        expect(modalIsVisible.value).toBe(true);
        onConfirm.value?.();
        expect(db_user_remove).toHaveBeenCalledOnce();
    });
    it("should add user", () => {
        const { handleAddUser } = useDbDetails(dbParams);
        handleAddUser();
        expect(modalIsVisible.value).toBe(true);
        setInputValue(KEY_MODAL, "username", "testUser");
        onConfirm.value?.();
        expect(db_user_add).toHaveBeenCalled();
    });
    it("should not add user if role is not admin", () => {
        const { handleAddUser } = useDbDetails(
            ref({ ...dbParams.value, role: "read" }),
        );
        handleAddUser();
        expect(modalIsVisible.value).toBe(false);
    });
    it("should not remove user if role is not admin", () => {
        const { handleRemoveUser } = useDbDetails(
            ref({ ...dbParams.value, role: "read" }),
        );
        handleRemoveUser("testUser");
        expect(modalIsVisible.value).toBe(false);
    });
    it("should not add user if no username", () => {
        const { handleAddUser } = useDbDetails(dbParams);
        handleAddUser();
        expect(modalIsVisible.value).toBe(true);
        onConfirm.value?.();
        expect(db_user_add).not.toHaveBeenCalled();
    });
    it("should prefill username if clicked", () => {
        const { handleUsernameClick } = useDbDetails(dbParams);
        handleUsernameClick("testUser", "read");
        expect(modalIsVisible.value).toBe(true);
        expect(getInputValue(KEY_MODAL, "username")).toBe("testUser");
        expect(getInputValue(KEY_MODAL, "role")).toBe("read");
    });
    it("should not allow to prefill username if owner", () => {
        const { handleUsernameClick } = useDbDetails(dbParams);
        handleUsernameClick("testOwner", "admin");
        expect(modalIsVisible.value).toBe(false);
    });
    it("should not allow to prefill username if can't edit users", () => {
        const { handleUsernameClick } = useDbDetails(
            ref({ ...dbParams.value, role: "read" }),
        );
        handleUsernameClick("testUser", "read");
        expect(modalIsVisible.value).toBe(false);
    });
});
