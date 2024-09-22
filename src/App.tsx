import { invoke } from "@tauri-apps/api";
import { useState, useEffect } from "react";
import { Card } from "./components/ui/card";
import AddShortcutForm from "./components/AddShortcutForm";
import { buttonVariants } from "./components/ui/button";
import clsx from "clsx";
import ConnectWithQR from "./components/ConnectWithQR";

interface Shortcut {
  id?: number;
  keys: string;
}

function App() {
  const [shortcuts, setShortcuts] = useState<Shortcut[]>([]);
  const [editingShortcut, setEditingShortcut] = useState<Shortcut | null>(null); // Edit state
  const [isAddingShortcut, setIsAddingShortcut] = useState(false); // Track adding

  useEffect(() => {
    fetchShortcuts();
  }, []);

  const fetchShortcuts = async () => {
    try {
      const data = await invoke<Shortcut[]>("get_shortcuts_command");
      setShortcuts(data);
    } catch (error) {
      console.error("Error fetching shortcuts:", error);
    }
  };

  const addOrUpdateShortcut = async (shortcut: Shortcut) => {
    try {
      const duplicate = shortcuts.some(
        (existing) =>
          existing.keys === shortcut.keys && existing.id !== shortcut.id
      );
      if (!duplicate) {
        if (shortcut.id) {
          await invoke("update_shortcut", { shortcut });
        } else {
          await invoke("add_shortcut", {
            shortcut: { ...shortcut, id: Date.now() },
          });
        }
      }

      setEditingShortcut(null);
      setIsAddingShortcut(false);
      fetchShortcuts();
    } catch (error) {
      console.error("Error saving shortcut:", error);
    }
  };

  const deleteShortcut = async (id?: number) => {
    try {
      if (id) await invoke("delete_shortcut", { id });
      setEditingShortcut(null); // Clear after deletion
      fetchShortcuts();
    } catch (error) {
      console.error("Error deleting shortcut:", error);
    }
  };

  return (
    <div className="p-4">
      <h1 className="text-2xl flex justify-between items-center font-bold mb-4 gap-2">
        <svg
          width="23"
          height="25"
          viewBox="0 0 23 25"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M12.8191 12.7553C12.8191 13.5725 12.7425 14.3847 12.5892 15.1917C12.4462 15.9886 12.2164 16.7445 11.8997 17.4596C11.6362 18.0631 11.302 18.6241 10.897 19.1428C10.7408 19.3429 10.4972 19.4517 10.2433 19.4517V19.4517C9.52915 19.4517 9.09629 18.6245 9.40558 17.9809C9.49712 17.7904 9.58274 17.5962 9.66245 17.3983C9.9587 16.673 10.1834 15.917 10.3367 15.1304C10.4899 14.3336 10.5665 13.5368 10.5665 12.74C10.5665 11.9329 10.4899 11.131 10.3367 10.3342C10.1937 9.53736 9.96892 8.76608 9.66245 8.02033C9.5797 7.81898 9.49136 7.62172 9.39744 7.42856C9.08357 6.78302 9.51899 5.95166 10.2368 5.95166V5.95166C10.4946 5.95166 10.7416 6.06321 10.8986 6.26769C11.3028 6.79425 11.6365 7.36315 11.8997 7.97436C12.2164 8.70989 12.4462 9.48117 12.5892 10.2882C12.7425 11.0952 12.8191 11.9176 12.8191 12.7553Z"
            fill="#F6C000"
          />
          <path
            d="M3.32625 7.6543C4.26442 7.6543 5.03676 7.74491 5.64325 7.92614C6.24975 8.10737 6.69988 8.38876 6.99365 8.77029C7.28743 9.14229 7.43431 9.62875 7.43431 10.2297C7.43431 10.6398 7.36324 11.0023 7.22109 11.3171C7.07894 11.6223 6.88941 11.8751 6.6525 12.0754C6.41559 12.2661 6.15025 12.3901 5.85647 12.4474V12.5189C6.15972 12.5857 6.44876 12.7049 6.72357 12.8766C6.99839 13.0387 7.22109 13.282 7.39167 13.6063C7.57172 13.9306 7.66175 14.3598 7.66175 14.894C7.66175 15.514 7.50539 16.0481 7.19266 16.4964C6.88941 16.9447 6.45349 17.2881 5.8849 17.5266C5.31631 17.7555 4.64348 17.87 3.86641 17.87H0V7.6543H3.32625ZM3.48261 11.6461C4.02277 11.6461 4.40183 11.5507 4.61979 11.36C4.83775 11.1597 4.94673 10.8926 4.94673 10.5587C4.94673 10.2249 4.8188 9.97691 4.56293 9.81475C4.31654 9.6526 3.92327 9.57152 3.38311 9.57152H2.41651V11.6461H3.48261ZM2.41651 13.5061V15.9241H3.62476C4.19335 15.9241 4.58662 15.8097 4.80458 15.5807C5.03202 15.3518 5.14574 15.0514 5.14574 14.6794C5.14574 14.4504 5.09836 14.2501 5.00359 14.0784C4.90883 13.8972 4.74299 13.7589 4.50607 13.6635C4.27864 13.5586 3.96591 13.5061 3.5679 13.5061H2.41651Z"
            fill="black"
          />
          <path
            d="M16.9751 13.0207C16.9751 14.1104 16.8729 15.1933 16.6686 16.2693C16.4779 17.3317 16.1715 18.3397 15.7492 19.2931C15.3878 20.1209 14.9265 20.8889 14.3655 21.5969C14.1851 21.8246 13.9069 21.9492 13.6164 21.9492H13.2158C12.3989 21.9492 11.907 21.0063 12.2793 20.2792C12.4568 19.9327 12.6191 19.5768 12.7662 19.2114C13.1612 18.2443 13.4609 17.2364 13.6652 16.1876C13.8695 15.1252 13.9717 14.0627 13.9717 13.0003C13.9717 11.9242 13.8695 10.855 13.6652 9.79258C13.4745 8.73015 13.1749 7.70177 12.7662 6.70745C12.6126 6.33354 12.4445 5.97023 12.2619 5.6175C11.8865 4.89206 12.3785 3.94922 13.1953 3.94922H13.6114C13.9048 3.94922 14.1855 4.07642 14.3658 4.30781C14.9267 5.0274 15.3878 5.80685 15.7492 6.64615C16.1715 7.62686 16.4779 8.65523 16.6686 9.73129C16.8729 10.8073 16.9751 11.9038 16.9751 13.0207Z"
            fill="#FFD705"
          />
          <path
            d="M22.4914 12.8234C22.4914 14.2763 22.3552 15.7201 22.0827 17.1548C21.8285 18.5714 21.4199 19.9153 20.8569 21.1866C20.3546 22.3369 19.7079 23.4004 18.9166 24.3773C18.7338 24.603 18.4561 24.728 18.1656 24.728H16.8251C16.0083 24.728 15.5212 23.7896 15.9213 23.0774C16.2814 22.4363 16.6008 21.7697 16.8796 21.0776C17.4062 19.7882 17.8058 18.4443 18.0782 17.0458C18.3506 15.6293 18.4868 14.2127 18.4868 12.7961C18.4868 11.3614 18.3506 9.93575 18.0782 8.51917C17.8239 7.1026 17.4244 5.73143 16.8796 4.40567C16.5901 3.7014 16.2623 3.02532 15.896 2.37742C15.494 1.66635 15.981 0.728027 16.7979 0.728027H18.1606C18.454 0.728027 18.7341 0.855633 18.9169 1.08508C19.708 2.07801 20.3547 3.15763 20.8569 4.32394C21.4199 5.63155 21.8285 7.00271 22.0827 8.43745C22.3552 9.87218 22.4914 11.3342 22.4914 12.8234Z"
            fill="#FFEC87"
          />
        </svg>
        <ConnectWithQR></ConnectWithQR>
      </h1>
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
        {/* Add Shortcut Button */}
        <AddShortcutForm
          onSave={addOrUpdateShortcut}
          onClose={() => setIsAddingShortcut(false)} // Reset adding state
          existingShortcut={null}
          isOpen={isAddingShortcut} // Control form visibility for adding
          openForm={() => setIsAddingShortcut(true)} // Trigger form open
        />
        {shortcuts.map((shortcut) => (
          <Card
            key={shortcut.id}
            className={clsx(
              buttonVariants({ variant: "outline" }),
              "relative h-32 p-4 cursor-pointer"
            )}
            onClick={() => setEditingShortcut(shortcut)} // Open for editing
          >
            <div className="flex flex-col items-center justify-center h-full">
              <p className="text-sm text-gray-500">{shortcut.keys}</p>
            </div>
          </Card>
        ))}

        {/* Edit Shortcut Dialog */}
        {editingShortcut && (
          <AddShortcutForm
            onSave={addOrUpdateShortcut}
            onClose={() => setEditingShortcut(null)} // Reset edit state
            existingShortcut={editingShortcut} // Pass shortcut to edit
            onDelete={() => deleteShortcut(editingShortcut.id)} // Delete handler
            isOpen={!!editingShortcut} // Control edit dialog visibility
          />
        )}
      </div>
    </div>
  );
}

export default App;
