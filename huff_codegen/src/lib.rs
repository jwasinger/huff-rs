//! ## Codegen
//!
//! Code Generation Module for the Huff Language.
//!
//! #### Usage
//!
//! Below we showcase generating a compile artifact from compiled bytecode using `huff_codegen`.
//!
//! ```rust
//! use huff_codegen::*;
//!
//! // Instantiate an empty Codegen
//! let mut cg = Codegen::new();
//! assert!(cg.ast.is_none());
//! assert!(cg.artifact.is_none());
//!
//! // ERC20 Bytecode
//! let main_bytecode = "60003560E01c8063a9059cbb1461004857806340c10f19146100de57806370a082311461014e57806318160ddd1461016b578063095ea7b314610177578063dd62ed3e1461018e575b600435336024358160016000526000602001526040600020548082116100d8578190038260016000526000602001526040600020558281906001600052600060200152604060002054018360016000526000602001526040600020556000527fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60206000a3600160005260206000f35b60006000fd5b60005433146100ed5760006000fd5b600435600060243582819060016000526000602001526040600020540183600160005260006020015260406000205580600254016002556000527fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60206000a35b600435600160005260006020015260406000205460005260206000f35b60025460005260206000f35b602435600435336000526000602001526040600020555b60243560043560005260006020015260406000205460005260206000f3";
//! let constructor_bytecode = "33600055";
//! let inputs = vec![];
//! let churn_res = cg.churn(inputs, main_bytecode, constructor_bytecode);
//!
//! // Validate the output bytecode
//! assert_eq!(churn_res.unwrap().bytecode, "336000556101ac806100116000396000f360003560E01c8063a9059cbb1461004857806340c10f19146100de57806370a082311461014e57806318160ddd1461016b578063095ea7b314610177578063dd62ed3e1461018e575b600435336024358160016000526000602001526040600020548082116100d8578190038260016000526000602001526040600020558281906001600052600060200152604060002054018360016000526000602001526040600020556000527fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60206000a3600160005260206000f35b60006000fd5b60005433146100ed5760006000fd5b600435600060243582819060016000526000602001526040600020540183600160005260006020015260406000205580600254016002556000527fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60206000a35b600435600160005260006020015260406000205460005260206000f35b60025460005260206000f35b602435600435336000526000602001526040600020555b60243560043560005260006020015260406000205460005260206000f3");
//!
//! // Write the compile artifact out to a file
//! // cg.export("./output.json");
//! ```

#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![forbid(unsafe_code)]
#![forbid(where_clauses_object_safety)]

use huff_utils::{
    abi::*, artifact::*, ast::*, bytecode::*, error::CodegenError, prelude::CodegenErrorKind,
};
use std::fs;

/// ### Codegen
///
/// Code Generation Manager responsible for generating the code for the Huff Language.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Codegen<'a> {
    /// The Input AST
    pub ast: Option<Contract<'a>>,
    /// A cached codegen output artifact
    pub artifact: Option<Artifact>,
    /// Intermediate main bytecode store
    pub main_bytecode: Option<String>,
    /// Intermediate constructor bytecode store
    pub constructor_bytecode: Option<String>,
}

impl<'a> Codegen<'a> {
    /// Public associated function to instantiate a new Codegen instance.
    pub fn new() -> Self {
        Self { ast: None, artifact: None, main_bytecode: None, constructor_bytecode: None }
    }

    /// Generates main bytecode from a Contract AST
    ///
    /// # Arguments
    ///
    /// * `ast` - Optional Contract Abstract Syntax Tree
    pub fn roll(&mut self, ast: Option<Contract<'a>>) -> Result<String, CodegenError> {
        let mut bytecode: String = String::default();

        // Grab the AST
        let contract: &Contract<'a> = match &ast {
            Some(a) => a,
            None => match &self.ast {
                Some(a) => a,
                None => {
                    tracing::error!("Neither Codegen AST was set nor passed in as a parameter to Codegen::roll()!");
                    return Err(CodegenError {
                        kind: CodegenErrorKind::MissingAst,
                        span: None,
                        token: None,
                    })
                }
            },
        };

        // TODO: main logic to create the main contract bytecode

        // Set bytecode and return
        if self.main_bytecode.is_none() {
            self.main_bytecode = Some(bytecode.clone());
        }
        Ok(bytecode)
    }

    /// Gracefully get the Contract AST
    pub fn graceful_ast_grab(
        &self,
        ast: Option<Contract<'a>>,
    ) -> Result<Contract<'a>, CodegenError> {
        match ast {
            Some(a) => Ok(a),
            None => match &self.ast {
                Some(a) => Ok(a.clone()),
                None => {
                    tracing::error!("Neither Codegen AST was set nor passed in as a parameter to Codegen::construct()!");
                    return Err(CodegenError {
                        kind: CodegenErrorKind::MissingAst,
                        span: None,
                        token: None,
                    })
                }
            },
        }
    }

    /// Generates constructor bytecode from a Contract AST
    ///
    /// # Arguments
    ///
    /// * `ast` - Optional Contract Abstract Syntax Tree
    pub fn construct(&mut self, ast: Option<Contract<'a>>) -> Result<String, CodegenError> {
        // Grab the AST
        let contract: Contract<'a> = self.graceful_ast_grab(ast.clone())?;

        // Find the constructor macro
        let c_macro: MacroDefinition<'a> =
            if let Some(m) = contract.find_macro_by_name("CONSTRUCTOR") {
                m
            } else {
                tracing::error!("CONSTRUCTOR Macro definition missing in AST!");
                return Err(CodegenError {
                    kind: CodegenErrorKind::MissingConstructor,
                    span: None,
                    token: None,
                })
            };

        tracing::info!("Codegen found constructor macro: {:?}", c_macro);

        // For each MacroInvocation Statement, recurse into bytecode
        let recursed_bytecode: Vec<Byte> = self.recurse_bytecode(c_macro, ast)?;
        println!("Got recursed bytecode {:?}", recursed_bytecode);
        let bytecode = recursed_bytecode.iter().map(|byte| byte.0.to_string()).collect();
        println!("Final bytecode: {}", bytecode);

        // Return
        Ok(bytecode)
    }

    /// Recurses a MacroDefinition to generate Bytecode
    pub fn recurse_bytecode(
        &self,
        macro_def: MacroDefinition<'a>,
        ast: Option<Contract<'a>>,
    ) -> Result<Vec<Byte>, CodegenError> {
        let mut final_bytes: Vec<Byte> = vec![];

        println!("Recursing... {}", macro_def.name);

        // Grab the AST
        let contract: Contract<'a> = self.graceful_ast_grab(ast.clone())?;

        // Generate the macro bytecode
        let irb = macro_def.to_irbytecode()?;
        println!("Got IRBytecode: {:?}", irb);

        for irbyte in irb.0.clone().iter() {
            match irbyte {
                IRByte::Byte(b) => final_bytes.push(b.clone()),
                IRByte::Constant(name) => {
                    let constant = if let Some(m) = contract
                        .constants
                        .iter()
                        .filter(|const_def| const_def.name == *name)
                        .cloned()
                        .collect::<Vec<ConstantDefinition>>()
                        .get(0)
                    {
                        m.clone()
                    } else {
                        tracing::warn!("Failed to find macro \"{}\" in contract", name);

                        // TODO we should try and find the constant defined in other files here
                        return Err(CodegenError {
                            kind: CodegenErrorKind::MissingConstantDefinition,
                            span: None,
                            token: None,
                        })
                    };

                    println!("Found constant definition: {:?}", constant);

                    let push_bytes = match constant.value {
                        ConstVal::Literal(l) => {
                            let hex_literal: String = hex::encode(l);
                            format!("{:02x}{}", 95 + hex_literal.len() / 2, hex_literal)
                        }
                        ConstVal::FreeStoragePointer(_fsp) => {
                            // TODO: we need to grab the using the offset?
                            let offset: u8 = 0;
                            let hex_literal: String = hex::encode([offset]);
                            format!("{:02x}{}", 95 + hex_literal.len() / 2, hex_literal)
                        }
                    };
                    println!("Push bytes: {}", push_bytes);

                    final_bytes.push(Byte(push_bytes))
                }
                IRByte::Statement(s) => {
                    match s {
                        Statement::MacroInvocation(mi) => {
                            // Get the macro that matches this invocation and turn into bytecode
                            let ir_macro =
                                if let Some(m) = contract.find_macro_by_name(&mi.macro_name) {
                                    m
                                } else {
                                    // TODO: this is where the file imports must be resolved .. in
                                    // case macro definition is external
                                    tracing::warn!(
                                        "Invoked Macro \"{}\" not found in Contract",
                                        mi.macro_name
                                    );
                                    return Err(CodegenError {
                                        kind: CodegenErrorKind::MissingMacroDefinition,
                                        span: None,
                                        token: None,
                                    })
                                };

                            println!("Found inner macro: {}", ir_macro.name);
                            println!("{:?}", ir_macro);

                            // Recurse
                            let recursed_bytecode: Vec<Byte> = if let Ok(bytes) =
                                self.recurse_bytecode(ir_macro.clone(), ast.clone())
                            {
                                bytes
                            } else {
                                tracing::error!(
                                    "Codegen failed to recurse into macro {}",
                                    ir_macro.name
                                );
                                return Err(CodegenError {
                                    kind: CodegenErrorKind::FailedMacroRecursion,
                                    span: None,
                                    token: None,
                                })
                            };
                            final_bytes = final_bytes
                                .iter()
                                .cloned()
                                .chain(recursed_bytecode.iter().cloned())
                                .collect();
                        }
                        _ => {
                            tracing::error!("Codegen received unexpected Statement during Bytecode Construction!");
                            return Err(CodegenError {
                                kind: CodegenErrorKind::InvalidMacroStatement,
                                span: None,
                                token: None,
                            })
                        }
                    }
                }
            }
        }

        Ok(final_bytes)
    }

    /// Generate a codegen artifact
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of Tokens representing constructor arguments
    /// * `main_bytecode` - The compiled MAIN Macro bytecode
    /// * `constructor_bytecode` - The compiled `CONSTRUCTOR` Macro bytecode
    pub fn churn(
        &mut self,
        args: Vec<ethers::abi::token::Token>,
        main_bytecode: &str,
        constructor_bytecode: &str,
    ) -> Result<Artifact, CodegenError<'a>> {
        let mut artifact: &mut Artifact = if let Some(art) = &mut self.artifact {
            art
        } else {
            self.artifact = Some(Artifact::default());
            self.artifact.as_mut().unwrap()
        };

        let contract_length = main_bytecode.len() / 2;
        let constructor_length = constructor_bytecode.len() / 2;

        let contract_size = format!("{:04x}", contract_length);
        let contract_code_offset = format!("{:04x}", 13 + constructor_length);

        println!("Contract Size: {}", contract_size);
        println!("Contract Code Offset: {}", contract_code_offset);

        // Encode tokens as hex strings using ethers-abi and hex crates
        let encoded: Vec<Vec<u8>> =
            args.iter().map(|tok| ethers::abi::encode(&[tok.clone()])).collect();
        let hex_args: Vec<String> = encoded.iter().map(|tok| hex::encode(tok.as_slice())).collect();
        let constructor_args = hex_args.join("");

        // Generate the final bytecode
        let bootstrap_code = format!("61{}8061{}6000396000f3", contract_size, contract_code_offset);
        let constructor_code = format!("{}{}", constructor_bytecode, bootstrap_code);
        artifact.bytecode = format!("{}{}{}", constructor_code, main_bytecode, constructor_args);
        artifact.runtime = main_bytecode.to_string();
        Ok(artifact.clone())
    }

    /// Export
    ///
    /// Writes a Codegen Artifact out to the specified file.
    ///
    /// # Arguments
    ///
    /// * `out` - Output location to write the serialized json artifact to.
    pub fn export(&self, output: String) -> Result<(), CodegenError> {
        if let Some(art) = &self.artifact {
            let serialized_artifact = serde_json::to_string(art).unwrap();
            fs::write(output, serialized_artifact).expect("Unable to write file");
        } else {
            tracing::warn!(
                "Failed to export the compile artifact to the specified output location {}!",
                output
            );
        }
        Ok(())
    }

    /// Abi Generation
    ///
    /// Generates an ABI for the given Ast.
    /// Stores the generated ABI in the Codegen `artifact`.
    ///
    /// # Arguments
    ///
    /// * `ast` - The Contract Abstract Syntax Tree
    /// * `output` - An optional output path
    pub fn abigen(
        &mut self,
        ast: Contract<'a>,
        output: Option<String>,
    ) -> Result<Abi, CodegenError> {
        let abi: Abi = ast.into();

        // Set the abi on self
        match &mut self.artifact {
            Some(artifact) => {
                artifact.abi = Some(abi.clone());
            }
            None => {
                self.artifact = Some(Artifact { abi: Some(abi.clone()), ..Default::default() });
            }
        }

        // If an output's specified, write the artifact out
        if let Some(o) = output {
            if self.export(o).is_err() {
                // !! We should never get here since we set the artifact above !! //
            }
        }

        // Return the abi
        Ok(abi)
    }
}
