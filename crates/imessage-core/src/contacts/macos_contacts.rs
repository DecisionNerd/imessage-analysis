use std::collections::HashMap;

use crate::error::Result;

#[cfg(target_os = "macos")]
pub fn fetch() -> Result<HashMap<String, String>> {
    use std::cell::RefCell;
    use std::ptr::NonNull;

    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, Bool, ProtocolObject};
    use objc2::ClassType;
    use objc2_contacts::{
        CNContact, CNContactEmailAddressesKey, CNContactFamilyNameKey,
        CNContactFetchRequest, CNContactGivenNameKey, CNContactPhoneNumbersKey,
        CNContactStore, CNKeyDescriptor,
    };
    use objc2_foundation::{NSArray, NSString};

    unsafe {
        let store = CNContactStore::new();

        // Wrap each static key in a Retained by casting through *mut AnyObject.
        // The keys are static, so retain is safe and won't be deallocated.
        fn retain_key(key: &'static NSString) -> Retained<ProtocolObject<dyn CNKeyDescriptor>> {
            unsafe {
                let ptr: *mut AnyObject = key as *const NSString as *mut NSString as *mut AnyObject;
                let retained: Retained<AnyObject> = Retained::retain(ptr).unwrap();
                std::mem::transmute(retained)
            }
        }

        let keys_vec: Vec<Retained<ProtocolObject<dyn CNKeyDescriptor>>> = vec![
            retain_key(CNContactGivenNameKey),
            retain_key(CNContactFamilyNameKey),
            retain_key(CNContactPhoneNumbersKey),
            retain_key(CNContactEmailAddressesKey),
        ];
        let keys_array = NSArray::from_vec(keys_vec);

        let request = CNContactFetchRequest::initWithKeysToFetch(
            CNContactFetchRequest::alloc(),
            &keys_array,
        );

        // Use RefCell so we can mutate inside the Fn closure
        let entries: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());

        let block = block2::StackBlock::new(
            |contact: NonNull<CNContact>, _stop: NonNull<Bool>| {
                let contact = unsafe { contact.as_ref() };
                let given = unsafe { contact.givenName().to_string() };
                let family = unsafe { contact.familyName().to_string() };
                let name = format!("{given} {family}").trim().to_string();
                if name.is_empty() {
                    return;
                }
                let phones = unsafe { contact.phoneNumbers() };
                for labeled in phones.iter() {
                    let raw = unsafe { labeled.value().stringValue().to_string() };
                    entries.borrow_mut().push((normalize_phone(&raw), name.clone()));
                }
                let emails = unsafe { contact.emailAddresses() };
                for labeled in emails.iter() {
                    let email = unsafe { labeled.value().to_string().to_lowercase() };
                    entries.borrow_mut().push((email, name.clone()));
                }
            },
        );

        let mut error = None;
        let ok = store.enumerateContactsWithFetchRequest_error_usingBlock(
            &request,
            Some(&mut error),
            &block,
        );

        if !ok || error.is_some() {
            tracing::warn!(
                "Contacts.app access denied or unavailable — \
                 name resolution will use config file only"
            );
            return Ok(HashMap::new());
        }

        Ok(entries.into_inner().into_iter().collect())
    }
}

fn normalize_phone(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    for (i, c) in raw.chars().enumerate() {
        if c == '+' && i == 0 {
            result.push(c);
        } else if c.is_ascii_digit() {
            result.push(c);
        }
    }
    result
}

#[cfg(not(target_os = "macos"))]
pub fn fetch() -> Result<HashMap<String, String>> {
    Ok(HashMap::new())
}
