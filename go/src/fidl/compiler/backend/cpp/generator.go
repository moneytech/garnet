// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package cpp

import (
	"fidl/compiler/backend/cpp/ir"
	"fidl/compiler/backend/cpp/templates"
	"fidl/compiler/backend/types"
	"os"
	"path/filepath"
	"text/template"
)

type FidlGenerator struct{}

const ownerReadWriteNoExecute = 0644

func writeFile(outputFilename string,
	templateName string,
	tmpls *template.Template,
	tree ir.Root) error {
	f, err := os.Create(outputFilename)
	if err != nil {
		return err
	}
	defer f.Close()
	return tmpls.ExecuteTemplate(f, templateName, tree)
}

func (_ FidlGenerator) GenerateFidl(
	data types.Root, _args []string,
	outputDir string, srcRootPath string) error {

	tree := ir.Compile(data)

	parentDir := filepath.Join(outputDir, srcRootPath)
	err := os.MkdirAll(parentDir, ownerReadWriteNoExecute)
	if err != nil {
		return err
	}

	tmpls := template.New("CPPTemplates")
	template.Must(tmpls.Parse(templates.Enum))
	template.Must(tmpls.Parse(templates.Header))
	template.Must(tmpls.Parse(templates.Implementation))
	template.Must(tmpls.Parse(templates.Interface))
	template.Must(tmpls.Parse(templates.Struct))
	template.Must(tmpls.Parse(templates.Union))

	outputFilename := filepath.Join(parentDir, "generated.h")
	err = writeFile(outputFilename, "GenerateHeaderFile", tmpls, tree)
	if err != nil {
		return err
	}

	outputFilename = filepath.Join(parentDir, "generated.cc")
	err = writeFile(outputFilename, "GenerateImplementationFile", tmpls, tree)
	if err != nil {
		return err
	}

	return nil
}
