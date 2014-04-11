/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::DOMImplementationBinding;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::{JS, JSRef, RootCollection, Unrooted, OptionalRootable};
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::bindings::error::{Fallible, InvalidCharacter, NamespaceError};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::document::{Document, HTMLDocument, NonHTMLDocument, DocumentMethods};
use dom::documenttype::DocumentType;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::node::{Node, NodeMethods};
use dom::text::Text;
use dom::window::{Window, WindowMethods};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct DOMImplementation {
    pub owner: JS<Window>,
    pub reflector_: Reflector,
}

impl DOMImplementation {
    pub fn new_inherited(owner: JS<Window>) -> DOMImplementation {
        DOMImplementation {
            owner: owner,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(owner: &JSRef<Window>) -> Unrooted<DOMImplementation> {
        reflect_dom_object(~DOMImplementation::new_inherited(owner.unrooted()), owner,
                           DOMImplementationBinding::Wrap)
    }
}

impl Reflectable for DOMImplementation {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

pub trait DOMImplementationMethods {
    fn CreateDocumentType(&self, qname: DOMString, pubid: DOMString, sysid: DOMString) -> Fallible<Unrooted<DocumentType>>;
    fn CreateDocument(&self, namespace: Option<DOMString>, qname: DOMString,
                      mut maybe_doctype: Option<JSRef<DocumentType>>) -> Fallible<Unrooted<Document>>;
    fn CreateHTMLDocument(&self, title: Option<DOMString>) -> Unrooted<Document>;
}

// http://dom.spec.whatwg.org/#domimplementation
impl<'a> DOMImplementationMethods for JSRef<'a, DOMImplementation> {
    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
    fn CreateDocumentType(&self, qname: DOMString, pubid: DOMString, sysid: DOMString) -> Fallible<Unrooted<DocumentType>> {
        let roots = RootCollection::new();
        match xml_name_type(qname) {
            // Step 1.
            InvalidXMLName => Err(InvalidCharacter),
            // Step 2.
            Name => Err(NamespaceError),
            // Step 3.
            QName => {
                let owner = self.owner.root(&roots);
                let document = owner.deref().Document().root(&roots);
                Ok(DocumentType::new(qname, Some(pubid), Some(sysid), &*document))
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocument
    fn CreateDocument(&self, namespace: Option<DOMString>, qname: DOMString,
                      mut maybe_doctype: Option<JSRef<DocumentType>>) -> Fallible<Unrooted<Document>> {
        let roots = RootCollection::new();
        let win = self.owner.root(&roots);

        // Step 1.
        let mut doc = Document::new(&win.root_ref(), None, NonHTMLDocument, None).root(&roots);
        // Step 2-3.
        let mut maybe_elem = if qname.is_empty() {
            None
        } else {
            match doc.CreateElementNS(namespace, qname) {
                Err(error) => return Err(error),
                Ok(elem) => Some(elem)
            }
        };

        {
            let doc_node: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut *doc);

            // Step 4.
            match maybe_doctype {
                None => (),
                Some(ref mut doctype) => {
                    assert!(doc_node.AppendChild(NodeCast::from_mut_ref(doctype)).is_ok())
                }
            }

            // Step 5.
            match maybe_elem.root(&roots) {
                None => (),
                Some(mut elem) => {
                    assert!(doc_node.AppendChild(NodeCast::from_mut_ref(&mut *elem)).is_ok())
                }
            }
        }

        // Step 6.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 7.
        Ok(Unrooted::new_rooted(&*doc))
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    fn CreateHTMLDocument(&self, title: Option<DOMString>) -> Unrooted<Document> {
        let roots = RootCollection::new();
        let owner = self.owner.root(&roots);

        // Step 1-2.
        let mut doc = Document::new(&owner.root_ref(), None, HTMLDocument, None).root(&roots);
        let mut doc_alias = doc.clone();
        let doc_node: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut doc_alias);

        {
            // Step 3.
            let mut doc_type = DocumentType::new(~"html", None, None, &*doc).root(&roots);
            assert!(doc_node.AppendChild(NodeCast::from_mut_ref(&mut *doc_type)).is_ok());
        }

        {
            // Step 4.
            let mut doc_html = NodeCast::from_unrooted(HTMLHtmlElement::new(~"html", &*doc)).root(&roots);
            assert!(doc_node.AppendChild(&mut *doc_html).is_ok());

            {
                // Step 5.
                let mut doc_head = NodeCast::from_unrooted(HTMLHeadElement::new(~"head", &*doc)).root(&roots);
                assert!(doc_html.AppendChild(&mut *doc_head).is_ok());

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let mut doc_title = NodeCast::from_unrooted(HTMLTitleElement::new(~"title", &*doc)).root(&roots);
                        assert!(doc_head.AppendChild(&mut *doc_title).is_ok());

                        // Step 6.2.
                        let mut title_text = Text::new(title_str, &*doc).root(&roots);
                        assert!(doc_title.AppendChild(NodeCast::from_mut_ref(&mut *title_text)).is_ok());
                    }
                }
            }

            // Step 7.
            let mut doc_body = HTMLBodyElement::new(~"body", &*doc).root(&roots);
            assert!(doc_html.AppendChild(NodeCast::from_mut_ref(&mut *doc_body)).is_ok());
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        Unrooted::new_rooted(&*doc)
    }
}
